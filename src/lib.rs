// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::all)]

use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, USER_AGENT};
use reqwest::{Client, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{value::from_value, Value};

pub trait Canonicalizable {
    fn canonicalize(&self, path: Vec<Link>) -> Vec<Link> {
        path
    }
}

impl<T: Canonicalizable> Canonicalizable for Vec<T> {}
impl<T: Canonicalizable> Canonicalizable for Option<T> {
    fn canonicalize(&self, path: Vec<Link>) -> Vec<Link> {
        match self {
            Some(t) => t.canonicalize(path),
            None => path,
        }
    }
}

pub trait Queriable {
    const QUERY: &'static [&'static str];
    const TYPE: &'static str;
}

impl<T: Queriable> Queriable for Vec<T> {
    const QUERY: &'static [&'static str] = T::QUERY;
    const TYPE: &'static str = T::TYPE;
}

impl<T: Queriable> Queriable for Option<T> {
    const QUERY: &'static [&'static str] = T::QUERY;
    const TYPE: &'static str = T::TYPE;
}

#[derive(Debug)]
pub struct HttpError(StatusCode);
impl std::error::Error for HttpError {}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct GraphQlError(Value);
impl std::error::Error for GraphQlError {}
impl std::fmt::Display for GraphQlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Link {
    name: String,
    kind: Option<String>,
    args: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
struct Core {
    http: Client,
    head: HeaderMap,
    base: Url,
}

#[derive(Serialize)]
struct Body {
    query: String,
}

pub struct Handle<T> {
    core: Arc<Core>,
    path: Vec<Link>,
    item: T,
}

impl<T: Canonicalizable + Queriable + DeserializeOwned> Handle<T> {
    pub async fn query<'a, U>(&self, name: &str, args: &[(&str, &str)]) -> anyhow::Result<Handle<U>>
    where
        U: DeserializeOwned,
        U: Canonicalizable,
        U: Queriable,
    {
        let mut body = Body {
            query: U::QUERY.join(" "),
        };

        let path = [Link {
            name: name.into(),
            kind: Some(U::TYPE.into()),
            args: args.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
        }];

        for link in path.iter().rev().chain(self.path.iter().rev()) {
            if let Some(kind) = link.kind.as_ref() {
                body.query = format!("... on {} {{ {} }}", kind, body.query);
            }

            let mut args = Vec::new();
            for (k, v) in link.args.iter() {
                args.push(format!("{}: {}", k, v));
            }

            body.query = if args.is_empty() {
                format!("{} {{ {} }}", &link.name, body.query)
            } else {
                format!("{}({}) {{ {} }}", &link.name, args.join(", "), body.query)
            }
        }

        let resp = self
            .core
            .http
            .post(self.core.base.clone())
            .headers(self.core.head.clone())
            .json(&body)
            .send()
            .await?;

        let code = resp.status();
        if code != StatusCode::OK {
            return Err(HttpError(code).into());
        }

        let mut root = resp.json::<Value>().await?;
        if root.get("errors").is_some() {
            return Err(GraphQlError(root).into());
        }

        let mut value = match root.get_mut("data") {
            Some(v) => v,
            None => return Err(GraphQlError(root).into()),
        };

        for link in self.path[1..].iter().chain(path.iter()) {
            value = match value.get_mut(&link.name) {
                Some(v) => v,
                None => return Err(GraphQlError(root).into()),
            };
        }

        let item: U = from_value(value.take())?;
        let path = item.canonicalize([&self.path[..], &path[..]].concat());

        Ok(Handle {
            core: self.core.clone(),
            path,
            item,
        })
    }
}

impl<T> std::ops::Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T> AsRef<T> for Handle<T> {
    fn as_ref(&self) -> &T {
        &self.item
    }
}

impl<T> std::borrow::Borrow<T> for Handle<T> {
    fn borrow(&self) -> &T {
        &self.item
    }
}

impl<T: Queriable + Default> Handle<T> {
    pub fn root(base: &str, headers: &[(&str, &str)]) -> anyhow::Result<Self> {
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

        let core = Arc::new(Core {
            http: Client::new(),
            base: base.parse()?,
            head,
        });

        Ok(Self {
            core,
            path: vec![Link {
                name: "query".into(),
                kind: Some(T::TYPE.into()),
                args: vec![],
            }],
            item: T::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::ser::to_string;

    pub trait Node {
        fn id(&self) -> &String;
    }

    impl<T: Queriable + Node> Canonicalizable for T {
        fn canonicalize(&self, _: Vec<Link>) -> Vec<Link> {
            let val = to_string(self.id()).unwrap();

            vec![
                Link {
                    name: "query".into(),
                    kind: None,
                    args: vec![],
                },
                Link {
                    name: "node".into(),
                    kind: Some(T::TYPE.into()),
                    args: vec![("id".into(), val)],
                },
            ]
        }
    }

    #[derive(Debug, Deserialize, Serialize, Default)]
    pub struct Query {}

    impl Canonicalizable for Query {}
    impl Queriable for Query {
        const QUERY: &'static [&'static str] = &[];
        const TYPE: &'static str = "Query";
    }

    impl Handle<Query> {
        pub async fn user(&self, login: &str) -> anyhow::Result<Handle<User>> {
            let login = to_string(login)?;
            self.query("user", &[("login", &login)]).await
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct User {
        pub id: String,
        pub login: String,
    }

    impl Node for User {
        fn id(&self) -> &String {
            &self.id
        }
    }

    impl Queriable for User {
        const QUERY: &'static [&'static str] = &["id", "login"];
        const TYPE: &'static str = "User";
    }

    impl Handle<User> {
        #[doc = "The user's description of what they're currently doing."]
        pub async fn status(&self) -> anyhow::Result<Handle<Option<UserStatus>>> {
            self.query("status", &[]).await
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct UserStatus {
        #[serde(rename = "indicatesLimitedAvailability")]
        pub indicates_limited_availability: bool,
    }

    impl Canonicalizable for UserStatus {}
    impl Queriable for UserStatus {
        const QUERY: &'static [&'static str] = &["indicatesLimitedAvailability"];
        const TYPE: &'static str = "UserStatus";
    }

    macro_rules! wait {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    fn root() -> Handle<Query> {
        let token = std::env::var("GITHUB_TOKEN");
        let token = token.expect("Missing `GITHUB_TOKEN` environment variable");
        let token = format!("token {}", token);

        let base = "https://api.github.com/graphql";

        Handle::<Query>::root(base, &[("Authorization", &token)]).unwrap()
    }

    #[test]
    fn query() {
        let query = root();

        let user = wait!(query.user("npmccallum")).unwrap();
        eprintln!("{:?}", *user);
    }

    #[test]
    fn nested() {
        let query = root();

        let user = wait!(query.user("npmccallum")).unwrap();
        eprintln!("{:?}", *user);

        let status = wait!(user.status()).unwrap();
        eprintln!("{:?}", *status);
    }
}
