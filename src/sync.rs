use crate::{
    beer::Database,
    onemorebeer::{Category, Repository as OneMoreBeer},
    untappd::Repository as Untappd,
};
use anyhow::Result;

pub(crate) struct Sync;

impl Sync {
    pub(crate) async fn run() -> Result<()> {
        let client = reqwest::Client::new();

        let mut beers = OneMoreBeer::load_category(&client, Category::Beer).await?;
        let mut ciders = OneMoreBeer::load_category(&client, Category::Cider).await?;
        let mut meads = OneMoreBeer::load_category(&client, Category::Mead).await?;

        Untappd::load_scores(&client, &mut beers).await?;
        Untappd::load_scores(&client, &mut ciders).await?;
        Untappd::load_scores(&client, &mut meads).await?;

        let db = Database {
            beers,
            ciders,
            meads,
        };
        db.write().await?;

        Ok(())
    }
}
