#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum PackedAs {
    Bottle,
    Can,
}

impl TryFrom<String> for PackedAs {
    type Error = ();

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match &s[..] {
            "butelka" => Ok(Self::Bottle),
            "puszka" => Ok(Self::Can),
            "keg" => Err(()),
            "beerbox" => Err(()),
            "0" => Err(()),
            _ => {
                panic!("Unknwon package type: {s}")
            }
        }
    }
}
