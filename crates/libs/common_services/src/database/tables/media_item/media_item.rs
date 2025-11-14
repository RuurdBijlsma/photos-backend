
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use utoipa::ToSchema;
use crate::database::media_item::capture_details::CaptureDetails;
use crate::database::media_item::details::Details;
use crate::database::media_item::gps::Gps;
use crate::database::media_item::panorama::Panorama;
use crate::database::media_item::time_details::TimeDetails;
use crate::database::media_item::weather::Weather;
use crate::database::visual_analysis::visual_analysis::VisualAnalysis;

/// The root struct representing a '`media_item`' and all its available, nested information.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct FullMediaItem {
    pub id: String,
    pub hash: String,
    pub relative_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_local: NaiveDateTime,
    pub taken_at_utc: Option<DateTime<Utc>>,
    pub use_panorama_viewer: bool,
    pub visual_analyses: Vec<VisualAnalysis>,
    pub gps: Option<Gps>,
    pub time_details: Option<TimeDetails>,
    pub weather: Option<Weather>,
    pub details: Option<Details>,
    pub capture_details: Option<CaptureDetails>,
    pub panorama: Option<Panorama>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct FullMediaItemRow {
    pub id: String,
    pub hash: String,
    pub relative_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_local: NaiveDateTime,
    pub taken_at_utc: Option<DateTime<Utc>>,
    pub use_panorama_viewer: bool,
    pub visual_analyses: Option<Json<Vec<VisualAnalysis>>>,
    pub gps: Option<Json<Gps>>,
    pub time_details: Option<Json<TimeDetails>>,
    pub weather: Option<Json<Weather>>,
    pub details: Option<Json<Details>>,
    pub capture_details: Option<Json<CaptureDetails>>,
    pub panorama: Option<Json<Panorama>>,
}

impl From<FullMediaItemRow> for FullMediaItem {
    fn from(r: FullMediaItemRow) -> Self {
        Self {
            id: r.id,
            hash: r.hash,
            relative_path: r.relative_path,
            created_at: r.created_at,
            updated_at: r.updated_at,
            width: r.width,
            height: r.height,
            is_video: r.is_video,
            duration_ms: r.duration_ms,
            taken_at_local: r.taken_at_local,
            taken_at_utc: r.taken_at_utc,
            use_panorama_viewer: r.use_panorama_viewer,
            visual_analyses: r.visual_analyses.map(|j| j.0).unwrap_or_default(),
            gps: r.gps.map(|j| j.0),
            time_details: r.time_details.map(|j| j.0),
            weather: r.weather.map(|j| j.0),
            details: r.details.map(|j| j.0),
            capture_details: r.capture_details.map(|j| j.0),
            panorama: r.panorama.map(|j| j.0),
        }
    }
}
