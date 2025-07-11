use crate::{
    beer::{Beer, PackedAs},
    onemorebeer::{Category, Client, PageOptions},
};
use anyhow::{Context, Result};
use reqwest_middleware::ClientWithMiddleware;

pub(crate) struct Repository;

impl Repository {
    pub(crate) async fn load_category(
        client: &ClientWithMiddleware,
        category: Category,
    ) -> Result<Vec<Beer>> {
        let last_page = Client::get_pages_count(client, category).await?;
        let pages = (1..=last_page).collect::<Vec<_>>();

        async fn get_concurrently(
            pages: &[usize],
            client: &ClientWithMiddleware,
            category: Category,
        ) -> Result<Vec<String>> {
            pages
                .iter()
                .map(|page| {
                    Client::load_page(
                        client,
                        PageOptions {
                            page: *page,
                            category,
                        },
                    )
                })
                .collect::<futures::future::JoinAll<_>>()
                .await
                .into_iter()
                .collect::<anyhow::Result<Vec<_>>>()
        }

        let mut beers = vec![];
        let total_chunks = pages.len() / 5 + 1;
        for (idx, chunk) in pages.chunks(5).enumerate() {
            println!(
                "Processing onemorebeer[{:?}] chunk {}/{}",
                category,
                idx + 1,
                total_chunks
            );
            for raw in get_concurrently(chunk, client, category).await? {
                let page: Page = serde_json::from_str(&raw).context("invalid json")?;
                for item in page.items {
                    match Beer::try_from(item) {
                        Ok(beer) => beers.push(beer),
                        Err(err) => log::error!("{err:?}"),
                    }
                }
            }
        }
        Ok(beers)
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page {
    items: Vec<Item>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Item {
    name: String,
    url: String,
    attributes: Vec<Attribute>,
    manufacturer: Manufacturer,
}

impl Item {
    fn abv(&self) -> Option<f32> {
        self.attributes
            .iter()
            .find(|attr| attr.name_code == "moc")
            .map(|attr| attr.value_code.as_str())
            .and_then(|value| value.strip_suffix("%"))
            .and_then(|v| v.parse::<f32>().ok())
    }

    fn style(&self) -> String {
        self.attributes
            .iter()
            .find(|a| a.name_code == "styl")
            .map(|a| a.value_code.clone())
            .unwrap_or_else(|| String::from("unknown"))
    }

    fn packed_as(&self) -> Result<PackedAs> {
        let attr = self
            .attributes
            .iter()
            .find(|attr| attr.name_code == "rodzaj_opakowania")
            .context("no PackedAs data")?;

        PackedAs::try_from(attr.value_code.as_str())
    }
}

impl TryFrom<Item> for Beer {
    type Error = anyhow::Error;

    fn try_from(item: Item) -> Result<Beer> {
        Beer::builder()
            .style(item.style())
            .abv(item.abv())
            .packed_as(item.packed_as()?)
            .name(item.name)
            .url(item.url)
            .manufacturer(item.manufacturer.name)
            .build()
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Attribute {
    name_code: String,
    value_code: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manufacturer {
    name: String,
}
