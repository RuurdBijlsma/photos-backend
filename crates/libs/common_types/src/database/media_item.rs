use crate::database::visual_analysis::VisualAnalysis;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use sqlx::types::Json;
use utoipa::ToSchema;

/// Corresponds to the 'location' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Location {
    pub name: Option<String>,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub country_code: String,
    pub country_name: String,
}

/// Corresponds to the '`time_details`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct TimeDetails {
    pub timezone_name: Option<String>,
    pub timezone_offset_seconds: Option<i32>,
    pub source: Option<String>,
    pub source_details: Option<String>,
    pub source_confidence: Option<String>,
}

/// Corresponds to the 'weather' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Weather {
    pub temperature: Option<f32>,
    pub dew_point: Option<f32>,
    pub relative_humidity: Option<i32>,
    pub precipitation: Option<f32>,
    pub snow: Option<i32>,
    pub wind_direction: Option<i32>,
    pub wind_speed: Option<f32>,
    pub peak_wind_gust: Option<f32>,
    pub pressure: Option<f32>,
    pub sunshine_minutes: Option<i32>,
    pub condition: Option<String>,
    pub sunrise: Option<DateTime<Utc>>,
    pub sunset: Option<DateTime<Utc>>,
    pub dawn: Option<DateTime<Utc>>,
    pub dusk: Option<DateTime<Utc>>,
    pub is_daytime: Option<bool>,
}

/// Corresponds to the 'details' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Details {
    pub mime_type: String,
    pub size_bytes: i64,
    pub is_motion_photo: bool,
    pub motion_photo_presentation_timestamp: Option<i64>,
    pub is_hdr: bool,
    pub is_burst: bool,
    pub burst_id: Option<String>,
    pub capture_fps: Option<f32>,
    pub video_fps: Option<f32>,
    pub is_nightsight: bool,
    pub is_timelapse: bool,
    pub exif: Option<Value>,
}

/// Corresponds to the '`capture_details`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct CaptureDetails {
    pub iso: Option<i32>,
    pub exposure_time: Option<f32>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

/// Corresponds to the 'panorama' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Panorama {
    pub is_photosphere: bool,
    pub projection_type: Option<String>,
    pub horizontal_fov_deg: Option<f32>,
    pub vertical_fov_deg: Option<f32>,
    pub center_yaw_deg: Option<f32>,
    pub center_pitch_deg: Option<f32>,
}

/// A composite struct representing data from the 'gps' table, with its associated 'location' data nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Gps {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub image_direction: Option<f64>,
    pub location: Option<Location>,
}

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
