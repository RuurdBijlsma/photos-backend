use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultItem {
    pub id: String,
    pub score: f32,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub items: Vec<SearchResultItem>,
}