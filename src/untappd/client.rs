use anyhow::{Context as _, Result};

use crate::{beer::Beer, cache::Cache};

pub(crate) struct Client;

const CACHE_NS: &str = "untappd";

fn cache_key(beer: &Beer) -> String {
    beer.url.strip_prefix('/').unwrap().to_string()
}

pub(crate) enum ValueOrRetryAfter<T> {
    Value(T),
    RetryAfter(u64),
}

impl Client {
    async fn search_one(
        client: &reqwest::Client,
        beer_name: &str,
    ) -> Result<ValueOrRetryAfter<String>> {
        let res = client
            .get("https://untappd.com/search")
            .query(&[("q", beer_name), ("type", "beer"), ("sort", "all")])
            .send()
            .await?;

        if res.status() == 429 {
            log::error!("Untappd returned 429");

            let retry_after = res
                .headers()
                .get("retry-after")
                .context("untappd response doesn't contain retry-after header")?
                .to_str()
                .context("non-utf8 retry-after header value in Untappd response")?
                .parse::<u64>()
                .context("non-numeric retry-after header value in Untappd response")?;
            return Ok(ValueOrRetryAfter::RetryAfter(retry_after));
        }

        let html = res.text().await?;

        Ok(ValueOrRetryAfter::Value(html))
    }

    pub(crate) async fn search_one_cached(
        client: &reqwest::Client,
        beer: &Beer,
    ) -> Result<ValueOrRetryAfter<String>> {
        let cache_key = cache_key(beer);

        if let Some(html) = Cache::read(CACHE_NS, &cache_key).await {
            Ok(ValueOrRetryAfter::Value(html))
        } else {
            match Self::search_one(client, &beer.name).await? {
                ValueOrRetryAfter::Value(html) => {
                    Cache::write(CACHE_NS, &cache_key, &html).await?;
                    Ok(ValueOrRetryAfter::Value(html))
                }
                retry_after @ ValueOrRetryAfter::RetryAfter(_) => Ok(retry_after),
            }
        }
    }
}
