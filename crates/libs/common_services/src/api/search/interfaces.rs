use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<i64>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub media_type: Option<SearchMediaType>,
    pub sort_by: Option<SearchSortBy>,
    pub negative_query: Option<String>,
    pub country_code: Option<String>,
    pub face_name: Option<String>,
}

#[derive(Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchMediaType {
    #[default]
    All,
    Photo,
    Video,
}

#[derive(Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchSortBy {
    #[default]
    Relevancy,
    Date,
}
