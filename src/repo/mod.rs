use anyhow::*;
use druid::{Data, Lens};
use serde::export::Formatter;
use std::{fmt, sync::Arc};

mod fetcher;

#[derive(Data, Debug, Default, Lens, Clone)]
pub struct RepoId {
    owner: String,
    name: String,
}

impl RepoId {
    pub fn new(owner: String, name: String) -> Self {
        Self { owner, name }
    }

    pub fn parse<S>(s: S) -> anyhow::Result<Self>
    where
        S: ToString,
    {
        let s = s.to_string();
        match s.split("/").collect::<Vec<_>>().as_slice() {
            [owner, name] => Ok(RepoId {
                owner: owner.to_string(),
                name: name.to_string(),
            }),
            _ => Err(anyhow!(
                "invalid format - expected: `owner/name`- received: '{}'",
                s
            )),
        }
    }
}

impl fmt::Display for RepoId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

pub type Repos = Vec<Repo>;

#[derive(Data, Debug, Default, Lens, Clone)]
pub struct Repo {
    pub repo_id: RepoId,
    pub description: Option<String>,
    pub lang: String,
    pub watchers: i64,
    pub stargazers: i64,
    pub forks: i64,
    pub open_issues: i64,
    pub open_pull_requests: i64,
}

pub type Releases = Vec<Release>;
pub type Assets = Vec<Asset>;

#[derive(Data, Debug, Default, Lens, Clone)]
pub struct Release {
    pub name: Option<String>,
    pub tag_name: String,
    pub assets: Arc<Assets>,
}

#[derive(Data, Debug, Default, Lens, Clone)]
pub struct Asset {
    pub name: String,
    pub downloads: i64,
}

impl Repo {
    pub async fn fetch_repo<S>(token: S, repo_id: &RepoId) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let fetcher = fetcher::Fetcher::with_token(token);
        fetcher.fetch_repo(repo_id).await
    }

    pub async fn fetch_repos<S>(token: S, repo_ids: Vec<RepoId>) -> anyhow::Result<Repos>
    where
        S: AsRef<str>,
    {
        let fetcher = fetcher::Fetcher::with_token(token);
        fetcher.fetch_repos(repo_ids).await
    }

    pub async fn fetch_releases<S>(&self, token: S) -> anyhow::Result<Releases>
    where
        S: AsRef<str>,
    {
        let fetcher = fetcher::Fetcher::with_token(token);
        fetcher.fetch_releases(self).await
    }
}
