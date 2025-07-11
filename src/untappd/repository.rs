use crate::{
    beer::Beer,
    untappd::{Client, client::ValueOrRetryAfter},
};
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;

pub(crate) struct Repository;

impl Repository {
    pub(crate) async fn load_scores(
        client: &ClientWithMiddleware,
        beers: &mut [Beer],
    ) -> Result<()> {
        async fn try_fill_one(
            client: &ClientWithMiddleware,
            beer: &mut Beer,
        ) -> Result<ValueOrRetryAfter<()>> {
            let html = match Client::search_one(client, beer).await? {
                ValueOrRetryAfter::Value(html) => html,
                ValueOrRetryAfter::RetryAfter(retry_after) => {
                    return Ok(ValueOrRetryAfter::RetryAfter(retry_after));
                }
            };
            static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"data-rating="(.+)""#).unwrap());

            let rating = RE
                .captures_iter(&html)
                .filter_map(|m| m.get(1))
                .next()
                .and_then(|r| r.as_str().parse::<f32>().ok());

            beer.untappd_score = rating;

            Ok(ValueOrRetryAfter::Value(()))
        }

        async fn get_concurrently(
            client: &ClientWithMiddleware,
            beers: &mut [Beer],
        ) -> Result<ValueOrRetryAfter<()>> {
            let max_retry = beers
                .iter_mut()
                .map(|beer| try_fill_one(client, beer))
                .collect::<futures::future::JoinAll<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter_map(|value_or_retry| match value_or_retry {
                    ValueOrRetryAfter::Value(_) => None,
                    ValueOrRetryAfter::RetryAfter(retry_after) => Some(retry_after),
                })
                .max();

            let out = match max_retry {
                Some(retry_after) => ValueOrRetryAfter::RetryAfter(retry_after),
                None => ValueOrRetryAfter::Value(()),
            };

            Ok(out)
        }

        let total_chunks = beers.len() / 20 + 1;
        for (idx, chunk) in beers.chunks_mut(20).enumerate() {
            log::info!("Processing untappd chunk {}/{}", idx + 1, total_chunks);

            loop {
                let value_or_retry_after = get_concurrently(client, chunk).await?;

                match value_or_retry_after {
                    ValueOrRetryAfter::Value(_) => break,
                    ValueOrRetryAfter::RetryAfter(retry_after) => {
                        let retry_after = retry_after + 1;
                        log::error!("sleeping for {}s", retry_after);
                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                    }
                }
            }
        }

        Ok(())
    }
}
