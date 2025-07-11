use crate::config::Config;
use anyhow::Context as _;
use http::Extensions;
use reqwest_middleware::{
    Middleware, Next,
    reqwest::{Request, Response},
};

pub(crate) struct LocalCacheMiddleware {
    namespace: String,
}

impl LocalCacheMiddleware {
    pub(crate) fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for LocalCacheMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let headers = req.headers_mut();
        let cache_key = headers
            .remove("local-cache-key")
            .context("no local-cache-key header")?;
        let cache_key = cache_key
            .to_str()
            .context("non-utf8 local-cache-key header")?;

        let cache_dir = Config::global().cache_dir.as_str();
        let ns_dir = format!("{cache_dir}/{}", self.namespace);
        let path = format!("{ns_dir}/{cache_key}");

        if let Ok(contents) = tokio::fs::read_to_string(&path).await {
            log::info!("[cached] {cache_key}");
            let res = http::Response::builder()
                .body(contents)
                .context("failed to restore cached response")?;
            return Ok(res.into());
        } else {
            let res = next.run(req, extensions).await?;

            let status = res.status();
            let headers = res.headers().clone();
            let body = res.bytes().await.context("failed to ready body")?;

            log::info!("caching {cache_key}");

            if status.is_success() {
                std::fs::create_dir_all(ns_dir).context("failed to create cache dir")?;

                tokio::fs::write(path, &body)
                    .await
                    .context("failed to rwite cache")?
            }

            let mut res = http::Response::builder()
                .status(status)
                .body(body)
                .context("failed to re-construct response")?;
            *res.headers_mut() = headers;

            Ok(res.into())
        }
    }
}
