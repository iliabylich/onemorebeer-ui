use anyhow::{Context, Result};

use crate::config::Config;

pub(crate) struct Cache;

impl Cache {
    pub(crate) async fn read(ns: &str, key: impl AsRef<str>) -> Option<String> {
        let path = cache_key_to_path(ns, key);
        tokio::fs::read_to_string(path).await.ok()
    }

    pub(crate) async fn write(
        ns: &str,
        key: impl AsRef<str>,
        contents: impl AsRef<[u8]>,
    ) -> Result<()> {
        create_cache_dir_if_missing(ns);

        let path = cache_key_to_path(ns, key);

        tokio::fs::write(path, contents)
            .await
            .context("failed to Cache::write")
    }

    pub(crate) async fn fetch<F, Fut>(ns: &str, key: impl AsRef<str>, f: F) -> Result<String>
    where
        Fut: std::future::Future<Output = Result<String>>,
        F: FnOnce() -> Fut,
    {
        let key = key.as_ref();

        if let Some(contents) = Self::read(ns, key).await {
            Ok(contents)
        } else {
            let contents = f().await?;
            Self::write(ns, key, &contents).await?;
            Ok(contents)
        }
    }
}

fn cache_dir() -> &'static str {
    Config::global().cache_dir.as_str()
}

fn create_cache_dir_if_missing(ns: &str) {
    let dir = format!("{}/{}", cache_dir(), ns);
    std::fs::create_dir_all(dir).expect("failed to create cache dir")
}

fn cache_key_to_path(ns: &str, key: impl AsRef<str>) -> String {
    format!("{}/{}/{}", cache_dir(), ns, key.as_ref())
}
