use crate::{beer::Beer, config::Config};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Database {
    pub(crate) beers: Vec<Beer>,
    pub(crate) ciders: Vec<Beer>,
    pub(crate) meads: Vec<Beer>,
}

impl Database {
    pub(crate) async fn write(&self) -> Result<()> {
        let contents = serde_json::to_string(&self)?;
        tokio::fs::write(Self::path(), contents)
            .await
            .context("failed to write database.json")
    }

    pub(crate) async fn read() -> Result<Self> {
        let contents = tokio::fs::read_to_string(Self::path())
            .await
            .context("failed to read database.json")?;
        serde_json::from_str(&contents).context("failed to parse database.json")
    }

    fn path() -> String {
        format!("{}/{}", Config::global().cache_dir, "database.json")
    }
}
