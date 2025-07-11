use crate::beer::PackedAs;
use anyhow::{Context as _, Result};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Beer {
    pub(crate) name: String,
    pub(crate) style: String,
    pub(crate) abv: Option<f32>,
    pub(crate) url: String,
    pub(crate) manufacturer: String,
    pub(crate) packed_as: PackedAs,
    pub(crate) untappd_score: Option<f32>,
}

#[derive(Default)]
pub(crate) struct BeerBuilder {
    pub(crate) name: Option<String>,
    pub(crate) style: Option<String>,
    pub(crate) abv: Option<f32>,
    pub(crate) url: Option<String>,
    pub(crate) manufacturer: Option<String>,
    pub(crate) packed_as: Option<PackedAs>,
}

impl BeerBuilder {
    pub(crate) fn name(mut self, name: String) -> Self {
        self.name = Some(clean_name(name));
        self
    }
    pub(crate) fn style(mut self, style: String) -> Self {
        self.style = Some(style);
        self
    }
    pub(crate) fn abv(mut self, abv: Option<f32>) -> Self {
        self.abv = abv;
        self
    }
    pub(crate) fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }
    pub(crate) fn manufacturer(mut self, manufacturer: String) -> Self {
        self.manufacturer = Some(manufacturer);
        self
    }
    pub(crate) fn packed_as(mut self, packed_as: PackedAs) -> Self {
        self.packed_as = Some(packed_as);
        self
    }
    pub(crate) fn build(self) -> Result<Beer> {
        Ok(Beer {
            name: self.name.context("no name given")?,
            style: self.style.context("no style given")?,
            abv: self.abv,
            url: self.url.context("no url given")?,
            manufacturer: self.manufacturer.context("no manufacturer given")?,
            packed_as: self.packed_as.context("no packed_as given")?,
            untappd_score: None,
        })
    }
}

impl Beer {
    pub(crate) fn builder() -> BeerBuilder {
        BeerBuilder::default()
    }
}

fn clean_name(mut name: String) -> String {
    name = name
        .replace("BUT.", "")
        .replace("BUTELKA", "")
        .replace("PUSZKA", "")
        .replace("0%", "");

    static VOL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\d+([,.]\d+)? L").expect("invalid VOL_RE regex"));
    name = VOL_RE.replace_all(&name, "").to_string();

    if let Some((l, _)) = name.split_once("PROMOCJA") {
        name = l.trim().to_string();
    }

    static GRAVITY_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\d+([,.]\d+)?°").expect("invalid GRAVITY_RE regex"));
    name = GRAVITY_RE.replace_all(&name, "").to_string();

    static SP_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}").expect("invalid SP_RE regex"));
    name = SP_RE.replace_all(&name, " ").to_string();

    name.trim().to_string()
}
