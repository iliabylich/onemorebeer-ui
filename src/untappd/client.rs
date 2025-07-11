use crate::beer::Beer;
use anyhow::{Context as _, Result};
use reqwest_middleware::ClientWithMiddleware;

pub(crate) struct Client;

fn cache_key(beer: &Beer) -> String {
    beer.url.strip_prefix('/').unwrap().to_string()
}

pub(crate) enum ValueOrRetryAfter<T> {
    Value(T),
    RetryAfter(u64),
}

impl Client {
    pub(crate) async fn search_one(
        client: &ClientWithMiddleware,
        beer: &Beer,
    ) -> Result<ValueOrRetryAfter<String>> {
        let res = client
            .get("https://untappd.com/search")
            .header("local-cache-key", cache_key(beer))
            .query(&[("q", beer.name.as_str()), ("type", "beer"), ("sort", "all")])
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
}
