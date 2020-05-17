use chrono::{DateTime, Utc};
use druid::{Data, Lens};
use std::sync::Arc;

pub type GitHubStats = Vec<GitHubStat>;

#[derive(Clone, Debug, Data, Lens)]
pub struct GitHubStat {
    pub name: String,
    pub lang: String,
    pub scrape_ts: Arc<DateTime<Utc>>,
    pub watchers: i32,
    pub watchers_diff: i32,
    pub stargazers: i32,
    pub stargazers_diff: i32,
    pub forks: i32,
    pub forks_diff: i32,
    pub open_issues: i32,
    pub open_issues_diff: i32,
    pub open_pull_requests: i32,
    pub open_pull_requests_diff: i32,
    pub downloads: i32,
    pub downloads_diff: i32,
}

impl GitHubStat {
    pub fn from_row(row: &postgres::Row) -> Self {
        Self {
            name: row.get("name"),
            lang: row.get("lang"),
            scrape_ts: Arc::new(row.get("scrape_ts")),
            watchers: row.get("watchers"),
            watchers_diff: row.get("watchers_diff"),
            stargazers: row.get("stargazers"),
            stargazers_diff: row.get("stargazers_diff"),
            forks: row.get("forks"),
            forks_diff: row.get("forks_diff"),
            open_issues: row.get("open_issues"),
            open_issues_diff: row.get("open_issues_diff"),
            open_pull_requests: 0,
            open_pull_requests_diff: 0,
            downloads: row.get("dl_count_acc"),
            downloads_diff: row.get("dl_count_acc_diff"),
        }
    }
}
