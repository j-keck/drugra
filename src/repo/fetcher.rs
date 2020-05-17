use super::*;
use futures::{stream, StreamExt};
use graphql_client::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.public.graphql",
    query_path = "assets/repo-query.graphql",
    response_derives = "Debug"
)]
struct RepoQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.public.graphql",
    query_path = "assets/releases-query.graphql",
    response_derives = "Debug"
)]
struct ReleasesQuery;

#[derive(Clone)]
pub struct Fetcher {
    url: String,
    bearer_auth: String,
}

impl Fetcher {
    pub fn with_token<S>(token: S) -> Self
    where
        S: AsRef<str>,
    {
        let bearer_auth = format!("Bearer {}", token.as_ref());
        Self {
            url: "https://api.github.com/graphql".to_string(),
            bearer_auth,
        }
    }

    pub async fn fetch_repo(&self, repo_id: &RepoId) -> anyhow::Result<Repo> {
        let owner = repo_id.owner.to_string();
        let name = repo_id.name.to_string();
        let query = RepoQuery::build_query(repo_query::Variables {
            owner: owner.clone(),
            name: name.clone(),
        });

        let data: repo_query::RepoQueryRepository = surf::post(&self.url)
            .set_header("authorization".parse().unwrap(), &self.bearer_auth)
            .body_json(&query)?
            .await
            .map_err(|e| anyhow!("post failed: {}", e))?
            .body_json::<Response<repo_query::ResponseData>>()
            .await?
            .data
            .ok_or(anyhow!("'data' missing in response"))?
            .repository
            .ok_or(anyhow!("'repository' missing in response"))?;

        Ok(Repo {
            repo_id: RepoId::new(owner, name),
            description: data.description,
            lang: data.primary_language.map_or("".into(), |o| o.name),
            watchers: data.watchers.total_count,
            stargazers: data.stargazers.total_count,
            forks: data.fork_count,
            open_issues: data.issues.total_count,
            open_pull_requests: data.pull_requests.total_count,
        })
    }

    pub async fn fetch_repos(&self, repo_ids: Vec<RepoId>) -> anyhow::Result<Repos> {
        // fetch each repo
        let results: Vec<Result<Repo, anyhow::Error>> =
            stream::unfold(repo_ids.into_iter(), |mut iter| async {
                let repo_id = iter.next()?;
                Some((self.fetch_repo(&repo_id).await, iter))
            })
            .collect::<Vec<_>>()
            .await;

        // this is `sequence` (go from `Vec<Result<Repo>>` to `Result<Vec<Repo>>`)
        results.into_iter().collect()
    }

    pub async fn fetch_releases(&self, repo: &Repo) -> anyhow::Result<Releases> {
        let owner = repo.repo_id.owner.to_string();
        let name = repo.repo_id.name.to_string();
        let query = ReleasesQuery::build_query(releases_query::Variables {
            owner: owner.clone(),
            name: name.clone(),
        });

        let data: releases_query::ReleasesQueryRepository = surf::post(&self.url)
            .set_header("authorization".parse().unwrap(), &self.bearer_auth)
            .body_json(&query)?
            .await
            .map_err(|e| anyhow!("post failed: {}", e))?
            .body_json::<Response<releases_query::ResponseData>>()
            .await?
            .data
            .ok_or(anyhow!("'data' missing in response"))?
            .repository
            .ok_or(anyhow!("'repository' missing in response"))?;

        data.releases
            .nodes
            .as_ref()
            .ok_or(anyhow!("'releases.nodes' missing in response"))?
            .into_iter()
            .map(|d| {
                let d = d
                    .as_ref()
                    .ok_or(anyhow!("'as_ref' on releases.nodes failed"))?;

                let assets = d
                    .release_assets
                    .nodes
                    .as_ref()
                    .ok_or(anyhow!("'release_assets.nodes' missing in response"))?
                    .into_iter()
                    .map(|d| {
                        let d = d
                            .as_ref()
                            .ok_or(anyhow!("'as_ref' on release_assets.nodes failed"))?;
                        Ok(Asset {
                            name: d.name.to_string(),
                            downloads: d.download_count,
                        })
                    })
                    .collect::<anyhow::Result<Assets>>()?;

                Ok(Release {
                    name: d.name.clone(),
                    tag_name: d.tag_name.to_string(),
                    assets: Arc::new(assets),
                })
            })
            .collect()
    }
}
