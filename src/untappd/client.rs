use anyhow::Result;

use crate::{beer::Beer, cache::Cache};

pub(crate) struct Client;

const CACHE_NS: &str = "untappd";

fn cache_key(beer: &Beer) -> String {
    beer.url.strip_prefix('/').unwrap().to_string()
}

impl Client {
    async fn search_one(client: &reqwest::Client, beer_name: &str) -> Result<String> {
        let res = client
            .get("https://untappd.com/search")
            .query(&[("q", beer_name), ("type", "beer"), ("sort", "all")])
            .send()
            .await?;

        if res.status() == 429 {
            anyhow::bail!("Untappd returned 429");
        }

        let html = res.text().await?;

        Ok(html)
    }

    pub(crate) async fn search_one_cached(client: &reqwest::Client, beer: &Beer) -> Result<String> {
        Cache::fetch(CACHE_NS, cache_key(beer), || {
            Self::search_one(client, &beer.name)
        })
        .await
    }
}
