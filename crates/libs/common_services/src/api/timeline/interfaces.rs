use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, Debug, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Desc,
    Asc,
}

impl SortDirection {
    #[must_use]
    pub const fn as_sql(&self) -> &'static str {
        match self {
            Self::Desc => "DESC",
            Self::Asc => "ASC",
        }
    }
}

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimelineParams {
    #[serde(default)]
    pub sort: SortDirection,
}

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetMediaByMonthParams {
    /// "YYYY-MM-DD" strings.
    pub months: String,
    #[serde(default)]
    pub sort: SortDirection,
}
