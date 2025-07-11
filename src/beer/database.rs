use crate::{beer::Beer, cache::Cache};
use anyhow::Result;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Database {
    pub(crate) beers: Vec<Beer>,
    pub(crate) ciders: Vec<Beer>,
    pub(crate) meads: Vec<Beer>,
}

const CACHE_NS: &str = "top";
const CACHE_KEY: &str = "database.json";

impl Database {
    pub(crate) async fn write(&self) -> Result<()> {
        let json = serde_json::to_string(&self)?;
        Cache::write(CACHE_NS, CACHE_KEY, json).await?;

        Ok(())
    }

    pub(crate) async fn read() -> Result<Option<Self>> {
        if let Some(json) = Cache::read(CACHE_NS, CACHE_KEY).await {
            Ok(Some(serde_json::from_str(&json)?))
        } else {
            Ok(None)
        }
    }
}
