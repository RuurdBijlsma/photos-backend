use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<i64>,
    pub threshold: Option<f64>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultItem {
    pub id: String,
    pub is_video: bool,
    pub is_panorama: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_local: NaiveDateTime,
    pub ratio: f32,

    // Score breakdown
    pub fts_score: f32,
    pub vector_score: f32,
    pub combined_score: f32,
}
