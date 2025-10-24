// crates/api/src/routes/photos/interfaces.rs

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RandomPhotoResponse {
    pub media_id: String,
    pub themes: Option<Vec<Value>>,
}

// --- Structs for Media Grid ---

/// Represents a summary of media items for a given month and year.
#[derive(Serialize, ToSchema, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSummary {
    pub year: i32,
    pub month: i32,
    pub media_count: i64,
}

/// Defines the query parameters for requesting media by month(s).
#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetByMonthParam {
    /// "YYYY-MM" string.
    pub month: String,
}

/// Defines the query parameters for requesting media by month(s).
#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetMediaByMonthParams {
    /// "YYYY-MM" strings.
    pub months: String,
}

/// A data transfer object representing a single media item in the grid.
#[derive(Serialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    #[serde(rename = "i")]
    pub id: String,
    #[serde(rename = "v")]
    pub is_video: i32,
    #[serde(rename = "d")]
    pub duration_ms: Option<i64>,
    #[serde(rename = "p")]
    pub use_panorama_viewer: i32,
    #[serde(rename = "t")]
    pub taken_at_local: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonthGroupDto {
    pub month: String,
    pub media_items: Vec<MediaItemDto>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedMediaResponse {
    pub months: Vec<MonthGroupDto>,
}

// Ratio types
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonMonthlyPhotoRatios {
    pub ratios: Vec<f32>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonPhotoRatiosResponse {
    pub months: Vec<JsonMonthlyPhotoRatios>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct MonthlyRatiosDto {
    pub month: String,
    pub ratios: Vec<f32>,
}
