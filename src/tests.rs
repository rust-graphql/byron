use crate::{base, unions};

use serde::{Deserialize, Serialize};
use serde_json::ser::to_string;

pub trait Node {
    fn id(&self) -> &String;
}

impl<T: base::Queryable + Node> base::Canonicalizable for T {
    fn canonicalize(&self) -> Vec<base::Link> {
        let val = to_string(self.id()).unwrap();

        vec![base::Link {
            kind: Some(T::QUERY.kind.into()),
            indx: base::Index {
                name: "node".into(),
                args: vec![("id".into(), val)],
            },
        }]
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Query {}

impl base::Canonicalizable for Query {
    fn canonicalize(&self) -> Vec<base::Link> {
        vec![]
    }
}

impl base::Queryable for Query {
    const QUERY: base::Fragment = base::Fragment {
        kind: "Query",
        body: &[],
    };
}

impl base::Path<Query> {
    pub async fn user(&self, login: &str) -> anyhow::Result<base::Path<User>> {
        let index = base::Index {
            name: "user".into(),
            args: vec![("login".into(), to_string(&login)?)],
        };

        self.get(index).await
    }

    pub async fn repository(
        &self,
        name: &str,
        owner: &str,
    ) -> anyhow::Result<Option<base::Path<Repository>>> {
        let index = base::Index {
            name: "repository".into(),
            args: vec![
                ("owner".into(), to_string(&owner)?),
                ("name".into(), to_string(&name)?),
            ],
        };

        self.get(index).await
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

impl base::Queryable for User {
    const QUERY: base::Fragment = base::Fragment {
        kind: "User",
        body: &[
            base::Selection::Field("id"),
            base::Selection::Field("login"),
        ],
    };
}

impl base::Path<User> {
    #[doc = "The user's description of what they're currently doing."]
    pub async fn status(&self) -> anyhow::Result<Option<base::Path<UserStatus>>> {
        let index = base::Index {
            name: "status".into(),
            args: vec![],
        };

        let out: base::Path<Option<_>> = self.get(index).await?;
        Ok(out.into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserStatus {
    #[serde(rename = "indicatesLimitedAvailability")]
    pub indicates_limited_availability: bool,
}

impl base::Queryable for UserStatus {
    const QUERY: base::Fragment = base::Fragment {
        kind: "UserStatus",
        body: &[base::Selection::Field("indicatesLimitedAvailability")],
    };
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
}

impl Node for Repository {
    fn id(&self) -> &String {
        &self.id
    }
}

impl base::Queryable for Repository {
    const QUERY: base::Fragment = base::Fragment {
        kind: "Repository",
        body: &[base::Selection::Field("id"), base::Selection::Field("name")],
    };
}

impl base::Path<Repository> {
    pub async fn issue_or_pull_request(
        &self,
        number: isize,
    ) -> anyhow::Result<Option<IssueOrPullRequest>> {
        let index = base::Index {
            name: "issueOrPullRequest".into(),
            args: vec![("number".into(), to_string(&number)?)],
        };

        self.get(index).await
    }
}

unions! {
    #[derive(Debug)]
    pub enum IssueOrPullRequest as IssueOrPullRequest {
        Issue as Issue(Issue),
        PullRequest as PullRequest(PullRequest),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    pub id: String,
    pub number: isize,
    pub title: String,
}

impl Node for Issue {
    fn id(&self) -> &String {
        &self.id
    }
}

impl base::Queryable for Issue {
    const QUERY: base::Fragment = base::Fragment {
        kind: "Issue",
        body: &[
            base::Selection::Field("id"),
            base::Selection::Field("number"),
            base::Selection::Field("title"),
        ],
    };
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequest {
    pub id: String,
    pub number: isize,
    pub title: String,
}

impl Node for PullRequest {
    fn id(&self) -> &String {
        &self.id
    }
}

impl base::Queryable for PullRequest {
    const QUERY: base::Fragment = base::Fragment {
        kind: "PullRequest",
        body: &[
            base::Selection::Field("id"),
            base::Selection::Field("number"),
            base::Selection::Field("title"),
        ],
    };
}

macro_rules! wait {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

fn root() -> base::Path<Query> {
    let token = std::env::var("GITHUB_TOKEN");
    let token = token.expect("Missing `GITHUB_TOKEN` environment variable");
    let token = format!("token {}", token);

    let base = "https://api.github.com/graphql";

    base::Path::<Query>::new(base, &[("Authorization", &token)]).unwrap()
}

#[test]
fn user() {
    let query = root();

    let user = wait!(query.user("npmccallum")).unwrap();
    eprintln!("{:?}", *user);
}

#[test]
fn user_userstatus() {
    let query = root();

    let user = wait!(query.user("npmccallum")).unwrap();
    eprintln!("{:?}", *user);

    let status = wait!(user.status()).unwrap();
    eprintln!("{:?}", status);
}

#[test]
fn repository() {
    let query = root();

    let repo = wait!(query.repository("byron", "rust-graphql"))
        .unwrap()
        .unwrap();
    eprintln!("{:?}", *repo);
}

#[test]
fn repository_issue() {
    let query = root();

    let repo = wait!(query.repository("byron", "rust-graphql"))
        .unwrap()
        .unwrap();
    eprintln!("{:?}", *repo);

    match wait!(repo.issue_or_pull_request(1)).unwrap() {
        Some(IssueOrPullRequest::Issue(issue)) => eprintln!("{:?}", *issue),
        ipr => panic!("Invalid return value: {:?}", ipr),
    }
}

#[test]
fn repository_pullrequest() {
    let query = root();

    let repo = wait!(query.repository("byron", "rust-graphql"))
        .unwrap()
        .unwrap();
    eprintln!("{:?}", *repo);

    match wait!(repo.issue_or_pull_request(2)).unwrap() {
        Some(IssueOrPullRequest::PullRequest(pr)) => eprintln!("{:?}", *pr),
        ipr => panic!("Invalid return value: {:?}", ipr),
    }
}
