use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::types::JsonValue;

/// Represents a geographical location.
#[derive(Debug, sqlx::FromRow)]
pub struct Location {
    pub id: i32,
    pub name: Option<String>,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
}

/// Represents a single media item (photo or video) in the database.
#[derive(Debug, sqlx::FromRow)]
pub struct MediaItem {
    pub id: String,
    pub relative_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub width: i32,
    pub height: i32,
    pub is_video: bool,
    pub duration_ms: Option<i64>,
    pub taken_at_naive: Option<NaiveDateTime>,
    pub use_panorama_viewer: Option<bool>,
}

/// Represents GPS data associated with a media item.
#[derive(Debug, sqlx::FromRow)]
pub struct Gps {
    pub media_item_id: String,
    pub location_id: Option<i32>,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub image_direction: Option<f64>,
}

/// Represents detailed time information for a media item.
#[derive(Debug, sqlx::FromRow)]
pub struct TimeDetails {
    pub media_item_id: String,
    pub datetime_utc: Option<DateTime<Utc>>,
    pub timezone_name: Option<String>,
    pub timezone_offset_seconds: Option<i32>,
    pub source: Option<String>,
    pub source_details: Option<String>,
    pub source_confidence: Option<i32>,
}

/// Represents weather data associated with a media item.
#[derive(Debug, sqlx::FromRow)]
pub struct Weather {
    pub media_item_id: String,
    pub temperature: Option<f32>,
    pub dew_point: Option<f32>,
    pub relative_humidity: Option<f32>,
    pub precipitation: Option<f32>,
    pub snow: Option<f32>,
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

/// Represents technical details of a media item's file and format.
#[derive(Debug, sqlx::FromRow)]
#[allow(clippy::struct_excessive_bools)]
pub struct Details {
    pub media_item_id: String,
    pub is_motion_photo: bool,
    pub motion_photo_presentation_timestamp: Option<i64>,
    pub is_hdr: bool,
    pub is_burst: bool,
    pub burst_id: Option<String>,
    pub capture_fps: Option<f32>,
    pub video_fps: Option<f32>,
    pub is_nightsight: bool,
    pub is_timelapse: bool,
    pub mime_type: String,
    pub size_bytes: i64,
    pub exif: Option<JsonValue>, // Using sqlx::types::JsonValue for JSONB
}

/// Represents the camera and settings used to capture a media item.
#[derive(Debug, sqlx::FromRow)]
pub struct CaptureDetails {
    pub media_item_id: String,
    pub iso: Option<i32>,
    pub exposure_time: Option<String>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

/// Represents panorama-specific metadata for a media item.
#[derive(Debug, sqlx::FromRow)]
pub struct Panorama {
    pub media_item_id: String,
    pub is_photosphere: bool,
    pub projection_type: Option<String>,
    pub horizontal_fov_deg: Option<f32>,
    pub vertical_fov_deg: Option<f32>,
    pub center_yaw_deg: Option<f32>,
    pub center_pitch_deg: Option<f32>,
}
