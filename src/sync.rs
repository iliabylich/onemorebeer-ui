use crate::{
    beer::Database,
    local_cache_middleware::LocalCacheMiddleware,
    onemorebeer::{Category, Repository as OneMoreBeer},
    untappd::Repository as Untappd,
};
use anyhow::Result;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, reqwest};

pub(crate) struct Sync;

impl Sync {
    pub(crate) async fn run() -> Result<()> {
        let client = client_with_local_cache("onemorebeer");
        let mut beers = OneMoreBeer::load_category(&client, Category::Beer).await?;
        let mut ciders = OneMoreBeer::load_category(&client, Category::Cider).await?;
        let mut meads = OneMoreBeer::load_category(&client, Category::Mead).await?;

        let client = client_with_local_cache("untappd");
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

fn client_with_local_cache(namespace: &str) -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(LocalCacheMiddleware::new(namespace))
        .build()
}
