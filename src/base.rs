// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, USER_AGENT};
use reqwest::{Client, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::from_value;

pub use serde_json::Value;

#[derive(Debug)]
pub struct HttpError(StatusCode);
impl std::error::Error for HttpError {}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct GqlError(Value);
impl std::error::Error for GqlError {}
impl std::fmt::Display for GqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize)]
struct Body {
    query: String,
}

impl From<Fragment> for Body {
    fn from(value: Fragment) -> Self {
        let body = value
            .body
            .iter()
            .map(|x| match *x {
                Selection::Field(field) => field.into(),
                Selection::Fragment(frag) => Body::from(frag).query,
            })
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            query: format!("... on {} {{ {} }}", value.kind, body),
        }
    }
}

#[derive(Clone, Debug)]
struct Http {
    http: Client,
    head: HeaderMap,
    base: Url,
}

impl Http {
    fn new(url: &str, headers: &[(&str, &str)]) -> anyhow::Result<Self> {
        const PRODUCT: &str = env!("CARGO_PKG_NAME");
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        let mut head = HeaderMap::new();
        for (k, v) in headers {
            head.insert::<HeaderName>(k.parse()?, v.parse()?);
        }

        if !head.contains_key(USER_AGENT) {
            let ua = format!("{}/{}", PRODUCT, VERSION).parse().unwrap();
            head.insert(USER_AGENT, ua);
        }

        Ok(Self {
            http: Client::new(),
            base: url.parse()?,
            head,
        })
    }

    async fn post(&self, body: &Body) -> anyhow::Result<Value> {
        let resp = self
            .http
            .post(self.base.clone())
            .headers(self.head.clone())
            .json(&body)
            .send()
            .await?;

        let code = resp.status();
        if code != StatusCode::OK {
            return Err(HttpError(code).into());
        }

        Ok(resp.json::<Value>().await?)
    }
}

#[derive(Clone, Debug)]
pub struct Index {
    pub name: String,
    pub args: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Link {
    pub indx: Index,
    pub kind: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub struct Fragment {
    pub kind: &'static str,
    pub body: &'static [Selection],
}

#[derive(Copy, Clone, Debug)]
pub enum Selection {
    Fragment(Fragment),
    Field(&'static str),
}

pub trait Queryable {
    const QUERY: Fragment;
}

impl<T: Queryable> Queryable for Option<T> {
    const QUERY: Fragment = T::QUERY;
}

impl<T: Queryable> Queryable for Path<T> {
    const QUERY: Fragment = T::QUERY;
}

impl<T: Queryable> Queryable for Vec<T> {
    const QUERY: Fragment = T::QUERY;
}

pub trait Canonicalizable: Queryable {
    fn canonicalize(&self) -> Vec<Link>;
}

#[derive(Clone)]
pub struct Path<T> {
    http: Arc<Http>,
    path: Vec<Link>,
    item: T,
}

impl<T: Canonicalizable> Path<T> {
    pub fn absolute(self) -> Self {
        Self {
            http: self.http,
            path: self.item.canonicalize(),
            item: self.item,
        }
    }
}

impl<T: Canonicalizable + Default> Path<T> {
    pub fn new(url: &str, headers: &[(&str, &str)]) -> anyhow::Result<Self> {
        let item = T::default();

        Ok(Path {
            http: Arc::new(Http::new(url, headers)?),
            path: item.canonicalize(),
            item,
        })
    }
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Path<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Path")
            // DO NOT EXPOSE `core`, IT MAY CONTAIN PASSWORDS!
            .field("path", &self.path)
            .field("item", &self.item)
            .finish()
    }
}

impl<T> From<Path<Option<T>>> for Option<Path<T>> {
    fn from(value: Path<Option<T>>) -> Self {
        let Path { http, path, item } = value;

        match item {
            Some(item) => Some(Path { http, path, item }),
            None => None,
        }
    }
}

impl<T: Canonicalizable> From<Path<Vec<T>>> for Vec<Path<T>> {
    fn from(value: Path<Vec<T>>) -> Self {
        let Path {
            http,
            path: _,
            item,
        } = value;

        item.into_iter()
            .map(|x| Path {
                http: http.clone(),
                path: x.canonicalize(),
                item: x,
            })
            .collect()
    }
}

impl<T> Path<T> {
    pub async fn get<U: Queryable>(&self, index: Index) -> anyhow::Result<U>
    where
        Self: Decoder<U>,
    {
        let links = &[Link {
            kind: Some(U::QUERY.kind.into()),
            indx: index.clone(),
        }];

        let mut body = Body::from(U::QUERY);
        for link in links.iter().rev().chain(self.path.iter().rev()) {
            if let Some(kind) = link.kind.as_ref() {
                body.query = format!("... on {} {{ {} }}", kind, body.query);
            }

            let mut args = Vec::new();
            for (k, v) in link.indx.args.iter() {
                args.push(format!("{}: {}", k, v));
            }

            body.query = if args.is_empty() {
                format!("{} {{ {} }}", &link.indx.name, body.query)
            } else {
                format!(
                    "{}({}) {{ {} }}",
                    &link.indx.name,
                    args.join(", "),
                    body.query
                )
            }
        }

        body.query = format!("query {{ {} }}", body.query);
        eprintln!("{}", body.query);

        let mut root = self.http.post(&body).await?;
        if root.get("errors").is_some() {
            return Err(GqlError(root).into());
        }

        let mut value = match root.get_mut("data") {
            Some(v) => v,
            None => return Err(GqlError(root).into()),
        };

        for link in self.path.iter().chain(links.iter()) {
            value = match value.get_mut(&link.indx.name) {
                Some(v) => v,
                None => return Err(GqlError(root).into()),
            };
        }

        self.decode(index, value.take())
    }
}

pub trait Decoder<T> {
    fn decode(&self, indx: Index, value: Value) -> anyhow::Result<T>;
}

impl<T: DeserializeOwned + Queryable, U> Decoder<Path<T>> for Path<U> {
    fn decode(&self, indx: Index, value: Value) -> anyhow::Result<Path<T>> {
        let mut path = self.path.clone();

        path.push(Link {
            kind: Some(T::QUERY.kind.into()),
            indx,
        });

        Ok(Path {
            item: from_value(value)?,
            http: self.http.clone(),
            path,
        })
    }
}

impl<T, U> Decoder<Option<T>> for Path<U>
where
    Self: Decoder<T>,
{
    fn decode(&self, index: Index, value: Value) -> anyhow::Result<Option<T>> {
        Ok(match value {
            Value::Null => None,
            value => Some(self.decode(index, value)?),
        })
    }
}

impl<T: DeserializeOwned + Queryable + Canonicalizable, U> Decoder<Vec<Path<T>>> for Path<U> {
    fn decode(&self, index: Index, value: Value) -> anyhow::Result<Vec<Path<T>>> {
        let Path::<Vec<T>> {
            http,
            path: _,
            item,
        } = self.decode(index, value)?;

        Ok(item
            .into_iter()
            .map(|item| Path {
                path: item.canonicalize(),
                http: http.clone(),
                item,
            })
            .collect())
    }
}

#[macro_export]
macro_rules! unions {
    ($(
        $(#[$attr:meta])*
        $vis:vis enum $wire:ident as $name:ident {
            $(
                $(#[$vattr:meta])*
                $vwire:ident as $vname:ident($vtype:ty)
            ),*

            $(,)?
        }
    )*) => {
        $(
            $(#[$attr])*
            $vis enum $name {
                $(
                    $(#[$vattr])*
                    $vname(base::Path<$vtype>)
                ),*
            }

            impl base::Queryable for $name {
                const QUERY: base::Fragment = base::Fragment {
                    kind: stringify!($wire),
                    body: &[
                        base::Selection::Field("__typename"),
                        $( base::Selection::Fragment(<$vtype>::QUERY) ),*
                    ]
                };
            }

            impl<T> base::Decoder<$name> for base::Path<T> {
                fn decode(&self, index: base::Index, value: base::Value) -> anyhow::Result<$name> {
                    Ok(match value.get("__typename") {
                        Some(base::Value::String(x)) => match &**x {
                            $(
                                stringify!($vwire) => $name::$vname(self.decode(index, value)?),
                            )*

                            _ => todo!()
                        },

                        _ => todo!()
                    })
                }
            }
        )*
    };
}
