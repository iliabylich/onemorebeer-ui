use crate::{cache::Cache, onemorebeer::Category};
use anyhow::{Context, Result};

pub(crate) struct Client;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PageOptions {
    pub(crate) page: usize,
    pub(crate) category: Category,
}

const CACHE_KEY: &str = "onemorebeer";

impl PageOptions {
    fn to_url(self) -> String {
        const BASE_URL: &str = "https://api-prod.onecommerce.shop/api/v1/catalog/app/auth-optional/products/search/items";
        const SORT: &str = "sortCriteria=RANK_DESC";
        const FILTER: &str = "filtersQuery=availability:true";
        const PAGE_SIZE: &str = "pageSize=45";

        format!(
            "{BASE_URL}?category={category}&{SORT}&{FILTER}&{PAGE_SIZE}&pageNumber={page}",
            category = self.category.to_param(),
            page = self.page
        )
    }

    fn to_cache_key(self) -> String {
        format!("{}-{}", self.category.to_cache_key_part(), self.page)
    }
}

impl Client {
    async fn load_page(client: &reqwest::Client, options: PageOptions) -> Result<String> {
        let url = options.to_url();
        client
            .get(url)
            .header("One-Tenant", "pinta")
            .send()
            .await
            .context("failed to download page")?
            .text()
            .await
            .context("failed to extract text from the response")
    }

    pub(crate) async fn load_page_cached(
        client: &reqwest::Client,
        options: PageOptions,
    ) -> Result<String> {
        Cache::fetch(CACHE_KEY, options.to_cache_key(), || {
            Self::load_page(client, options)
        })
        .await
    }

    pub(crate) async fn get_pages_count(
        client: &reqwest::Client,
        category: Category,
    ) -> Result<usize> {
        let body = Self::load_page_cached(client, PageOptions { page: 1, category }).await?;

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Page {
            total_pages: usize,
        }
        let page: Page = serde_json::from_str(&body).context("failed to parse JSON")?;

        Ok(page.total_pages)
    }
}
