use anyhow::{Context, Result};

pub(crate) struct Cache;

impl Cache {
    pub(crate) async fn read(ns: &str, key: impl AsRef<str>) -> Result<Option<String>> {
        Ok(tokio::fs::read_to_string(cache_key_to_path(ns, key)?)
            .await
            .ok())
    }

    pub(crate) async fn write(
        ns: &str,
        key: impl AsRef<str>,
        contents: impl AsRef<[u8]>,
    ) -> Result<()> {
        create_cache_dir_if_missing(ns).unwrap();

        tokio::fs::write(cache_key_to_path(ns, key)?, contents)
            .await
            .context("failed to Cache::set")
    }

    pub(crate) async fn fetch<F, Fut>(ns: &str, key: impl AsRef<str>, f: F) -> Result<String>
    where
        Fut: std::future::Future<Output = Result<String>>,
        F: FnOnce() -> Fut,
    {
        let key = key.as_ref();

        if let Some(contents) = Self::read(ns, key).await? {
            Ok(contents)
        } else {
            let contents = f().await?;
            Self::write(ns, key, &contents).await?;
            Ok(contents)
        }
    }
}

fn cache_dir() -> Result<&'static str> {
    let config = crate::config::Config::global()?;
    Ok(config.cache_dir.as_str())
}

fn create_cache_dir_if_missing(ns: &str) -> Result<()> {
    let dir = format!("{}/{}", cache_dir()?, ns);
    std::fs::create_dir_all(dir).context("failed to create cache dir")
}

fn cache_key_to_path(ns: &str, key: impl AsRef<str>) -> Result<String> {
    Ok(format!("{}/{}/{}", cache_dir()?, ns, key.as_ref()))
}
