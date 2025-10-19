// crates/api/src/routes/photos/interfaces.rs

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct RandomPhotoResponse {
    pub media_id: String,
    pub themes: Option<Vec<Value>>,
}

/// Represents a single media item in the API response.
/// This is a lightweight DTO (Data Transfer Object) for the frontend.
#[derive(Serialize, ToSchema, FromRow, Debug)]
pub struct MediaItemDto {
    pub id: String,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub taken_at_naive: NaiveDateTime,
}

/// Represents a group of media items that were taken on the same day.
#[derive(Serialize, ToSchema, Debug)]
pub struct DayGroup {
    /// The date in "YYYY-MM-DD" format.
    pub date: String,
    pub media_items: Vec<MediaItemDto>,
}

/// The main response body for paginated media requests.
#[derive(Serialize, ToSchema, Debug)]
pub struct PaginatedMediaResponse {
    /// A list of day-grouped media items.
    pub days: Vec<DayGroup>,
    /// Indicates if there are more older photos to fetch.
    pub has_more_after: bool,
    /// Indicates if there are more recent photos to fetch.
    pub has_more_before: bool,
}

/// Query parameters for standard, cursor-based pagination.
#[derive(Deserialize, IntoParams, ToSchema)]
pub struct GetMediaParams {
    /// Fetch media items taken strictly before this UTC timestamp.
    pub before: Option<DateTime<Utc>>,
    /// Fetch media items taken strictly after this UTC timestamp.
    pub after: Option<DateTime<Utc>>,
    /// The maximum number of media items to return. Defaults to 100.
    pub limit: Option<u32>,
}

/// Query parameters for jumping to a specific date.
#[derive(Deserialize, IntoParams, ToSchema)]
pub struct GetMediaByDateParams {
    /// The target date to jump to in "YYYY-MM-DD" format.
    pub date: NaiveDate,
    /// The number of items to fetch before the target date. Defaults to 50.
    pub before_limit: Option<u32>,
    /// The number of items to fetch from the target date onwards. Defaults to 50.
    pub after_limit: Option<u32>,
}
