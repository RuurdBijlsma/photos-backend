use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum Variant {
    Monochrome,
    Neutral,
    TonalSpot,
    Vibrant,
    Expressive,
    Fidelity,
    Content,
    Rainbow,
    FruitSalad,
}

impl Variant {
    /// Converts the enum variant to its uppercase string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Monochrome => "MONOCHROME",
            Self::Neutral => "NEUTRAL",
            Self::TonalSpot => "TONAL_SPOT",
            Self::Vibrant => "VIBRANT",
            Self::Expressive => "EXPRESSIVE",
            Self::Fidelity => "FIDELITY",
            Self::Content => "CONTENT",
            Self::Rainbow => "RAINBOW",
            Self::FruitSalad => "FRUIT_SALAD",
        }
    }
}