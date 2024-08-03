use serde::Deserialize;
use tokio::sync::OnceCell;

use anyhow::{Context, Result};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    pub(crate) listen_on: u16,
    pub(crate) cache_dir: String,
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[cfg(debug_assertions)]
const CONFIG_PATH: &str = "config.json";

#[cfg(not(debug_assertions))]
const CONFIG_PATH: &str = "/etc/onemorebeer-ui.json";

const DEFAULT_CONFIG: &str = include_str!("../config.example.json");

impl Config {
    pub(crate) fn load() -> Result<()> {
        if !std::path::Path::new(CONFIG_PATH).exists() {
            std::fs::write(CONFIG_PATH, DEFAULT_CONFIG).unwrap()
        }

        let json = std::fs::read_to_string(CONFIG_PATH).context("Failed to open config file")?;
        let config: Config = serde_json::from_str(&json).context("Failed to parse config file")?;
        println!("Running with config {:?}", config);

        CONFIG.set(config).context("Config has already been loaded")
    }

    pub(crate) fn global() -> Result<&'static Config> {
        CONFIG.get().context("Config is not loaded")
    }
}
