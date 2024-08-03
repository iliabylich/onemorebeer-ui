#[derive(Debug, Clone, Copy)]
pub(crate) enum Category {
    Beer,
    Mead,
    Cider,
}

impl Category {
    pub(crate) fn to_param(self) -> &'static str {
        match self {
            Self::Beer => "Piwa",
            Self::Mead => "Miody_Pitne",
            Self::Cider => "Cydry_Wina",
        }
    }

    pub(crate) fn to_cache_key_part(self) -> &'static str {
        match self {
            Self::Beer => "beer",
            Self::Mead => "mead",
            Self::Cider => "cider",
        }
    }
}
