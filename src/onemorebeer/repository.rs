use crate::{
    beer::{Beer, PackedAs},
    onemorebeer::{Category, Client, PageOptions},
};
use anyhow::{Context, Result};

pub(crate) struct Repository;

impl Repository {
    pub(crate) async fn load_category(
        client: &reqwest::Client,
        category: Category,
    ) -> Result<Vec<Beer>> {
        let last_page = Client::get_pages_count(client, category).await?;
        let pages = (1..=last_page).collect::<Vec<_>>();

        async fn get_concurrently(
            pages: &[usize],
            client: &reqwest::Client,
            category: Category,
        ) -> Result<Vec<String>> {
            pages
                .iter()
                .map(|page| {
                    Client::load_page_cached(
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
                    if let Ok(beer) = Beer::try_from(item) {
                        beers.push(beer);
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
            .find(|a| a.name_code == "moc")
            .and_then(|a| a.value_code.parse::<usize>().ok())
            .map(|abv| abv as f32 / 10.0)
    }

    fn style(&self) -> String {
        self.attributes
            .iter()
            .find(|a| a.name_code == "styl")
            .map(|a| a.value_code.clone())
            .unwrap_or_else(|| String::from("unknown"))
    }

    fn packed_as(&self) -> Result<PackedAs, ()> {
        self.attributes
            .iter()
            .find(|a| a.name_code == "rodzaj_opakowania")
            .map(|a| a.value_code.clone())
            .and_then(|s| PackedAs::try_from(s).ok())
            .ok_or(())
    }
}

impl TryFrom<Item> for Beer {
    type Error = ();

    fn try_from(item: Item) -> Result<Beer, ()> {
        Ok(Beer::builder()
            .style(item.style())
            .abv(item.abv())
            .packed_as(item.packed_as()?)
            .name(item.name)
            .url(item.url)
            .manufacturer(item.manufacturer.name)
            .build())
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
