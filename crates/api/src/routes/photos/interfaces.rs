// crates/api/src/routes/photos/interfaces.rs

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct RandomPhotoResponse {
    pub media_id: String,
    pub themes: Option<Vec<Value>>,
}

// --- Structs for Media Grid ---

/// Represents a summary of media items for a given month and year.
#[derive(Serialize, ToSchema, sqlx::FromRow)]
pub struct TimelineSummary {
    pub year: i32,
    pub month: i32,
    pub media_count: i64,
}

/// Defines the query parameters for requesting media by month(s).
#[derive(Deserialize, IntoParams, ToSchema)]
pub struct GetMediaByMonthParams {
    /// A comma-separated list of "YYYY-MM" strings.
    pub months: String,
}

/// A data transfer object representing a single media item in the grid.
#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct MediaItemDto {
    #[serde(rename = "i")]
    pub id: String,
    #[serde(rename = "w")]
    pub width: i32,
    #[serde(rename = "h")]
    pub height: i32,
    #[serde(rename = "v")]
    pub is_video: bool,
    #[serde(rename = "d")]
    pub duration_ms: Option<i64>,
    #[serde(rename = "p")]
    pub use_panorama_viewer: bool,
    #[serde(rename = "t")]
    pub taken_at_local: NaiveDateTime,
}

/// Represents a group of media items that were taken on the same day.
#[derive(Serialize, ToSchema)]
pub struct DayGroup {
    pub date: String,
    pub media_items: Vec<MediaItemDto>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonthGroup {
    pub month: String,
    pub days: Vec<DayGroup>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedMediaResponse {
    pub months: Vec<MonthGroup>,
}