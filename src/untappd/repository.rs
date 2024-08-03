use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{beer::Beer, untappd::Client};

pub(crate) struct Repository;

impl Repository {
    pub(crate) async fn load_scores(client: &reqwest::Client, beers: &mut [Beer]) {
        async fn try_fill_one(client: &reqwest::Client, beer: &mut Beer) -> Result<()> {
            let html = Client::search_one_cached(client, &beer).await?;
            static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"data-rating="(.+)""#).unwrap());

            let rating = RE
                .captures_iter(&html)
                .filter_map(|m| m.get(1))
                .next()
                .and_then(|r| r.as_str().parse::<f32>().ok());

            beer.untappd_score = rating;

            Ok(())
        }

        async fn fill_one(client: &reqwest::Client, beer: &mut Beer) {
            if let Err(err) = try_fill_one(client, beer).await {
                eprintln!(
                    "Failed to load untappd scores for beer {:?}: {:?}",
                    beer.name, err
                );
            }
        }

        async fn get_concurrently(client: &reqwest::Client, beers: &mut [Beer]) {
            beers
                .iter_mut()
                .map(|beer| fill_one(client, beer))
                .collect::<futures::future::JoinAll<_>>()
                .await;
        }

        let total_chunks = beers.len() / 20 + 1;
        for (idx, chunk) in beers.chunks_mut(20).enumerate() {
            println!("Processing untappd chunk {}/{}", idx + 1, total_chunks);
            get_concurrently(client, chunk).await;
        }
    }
}
