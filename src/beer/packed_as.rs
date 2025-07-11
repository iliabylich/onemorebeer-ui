use anyhow::{Result, bail};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum PackedAs {
    Bottle,
    Can,
}

impl TryFrom<&str> for PackedAs {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        match s {
            "butelka" => Ok(Self::Bottle),
            "puszka" => Ok(Self::Can),
            other => bail!("unsupported PackedAs variant {other:?}"),
        }
    }
}
