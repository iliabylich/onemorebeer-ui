use anyhow::{Context, Result};

use crate::{
    beer::{Beer, Scores},
    ratebeer::Client,
};

pub(crate) struct Repository;

impl Repository {
    pub(crate) async fn load_scores(client: &reqwest::Client, beers: &mut [Beer]) {
        async fn try_fill_one(client: &reqwest::Client, beer: &mut Beer) -> Result<()> {
            let response = Client::search_one_cached(client, &beer).await?;
            let response = serde_json::from_str::<'_, serde_json::Value>(&response)
                .context("Ratebeer returned invalid JSON")?;

            fn extract(response: &serde_json::Value) -> Option<Scores> {
                let beer = response
                    .get("data")?
                    .get("results")?
                    .get("items")?
                    .get(0)?
                    .get("beer")?;

                let overall = beer.get("overallScore")?.as_f64()? as u8;
                let style = beer.get("styleScore")?.as_f64()? as u8;

                Some(Scores { overall, style })
            }

            if let Some(score) = extract(&response) {
                beer.overall_score = Some(score.overall);
                beer.style_score = Some(score.style);
            }

            Ok(())
        }

        async fn fill_one(client: &reqwest::Client, beer: &mut Beer) {
            if let Err(err) = try_fill_one(client, beer).await {
                eprintln!(
                    "Failed to load RB scores for beer {:?}: {:?}",
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
            println!("Processing ratebeer chunk {}/{}", idx + 1, total_chunks);
            get_concurrently(client, chunk).await;
        }
    }
}
