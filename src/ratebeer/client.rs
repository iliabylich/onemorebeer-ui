use anyhow::{Context, Result};

use crate::{beer::Beer, cache::Cache};

pub(crate) struct Client;

const CACHE_NS: &str = "ratebeer";

fn cache_key(beer: &Beer) -> String {
    beer.url.strip_prefix('/').unwrap().to_string()
}

impl Client {
    async fn search_one(client: &reqwest::Client, beer_name: &str) -> Result<String> {
        let query = serde_json::json!({
            "operationName": "SearchResultsBeer",
            "variables": {
                "query": beer_name,
                "order": "MATCH",
                "first": 1
            },
            "query": include_str!("query.gql")
        });

        let response = client
            .post("https://beta.ratebeer.com/v1/api/graphql/")
            .json(&query)
            .send()
            .await
            .context("failed to get gql Ratebeer data")?
            .text()
            .await
            .context("failed to read Ratebeer response")?;

        Ok(response)
    }

    pub(crate) async fn search_one_cached(client: &reqwest::Client, beer: &Beer) -> Result<String> {
        Cache::fetch(CACHE_NS, cache_key(beer), || {
            Self::search_one(client, &beer.name)
        })
        .await
    }
}
