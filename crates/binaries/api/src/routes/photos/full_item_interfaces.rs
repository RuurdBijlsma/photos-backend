use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the 'location' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: i32,
    pub name: Option<String>,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub country_code: String,
    pub country_name: String,
}

/// Corresponds to the 'ocr_box' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OcrBox {
    pub id: i64,
    pub ocr_data_id: i64,
    pub text: String,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

/// Corresponds to the 'face' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Face {
    pub id: i64,
    pub visual_analysis_id: i64,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: String,
    pub mouth_left_x: f32,
    pub mouth_left_y: f32,
    pub mouth_right_x: f32,
    pub mouth_right_y: f32,
    pub nose_tip_x: f32,
    pub nose_tip_y: f32,
    pub eye_left_x: f32,
    pub eye_left_y: f32,
    pub eye_right_x: f32,
    pub eye_right_y: f32,
    #[schema(value_type = Vec<f32>)]
    pub embedding: Vec<f32>,
}

/// Corresponds to the 'detected_object' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DetectedObject {
    pub id: i64,
    pub visual_analysis_id: i64,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub label: String,
}

/// Corresponds to the 'quality_data' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QualityData {
    pub visual_analysis_id: i64,
    pub blurriness: f64,
    pub noisiness: f64,
    pub exposure: f64,
    pub quality_score: f64,
}

/// Corresponds to the 'color_data' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColorData {
    pub visual_analysis_id: i64,
    pub themes: Vec<Value>,
    pub prominent_colors: Option<Vec<String>>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
    pub histogram: Option<Value>,
}

/// Corresponds to the 'caption_data' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaptionData {
    pub visual_analysis_id: i64,
    pub default_caption: Option<String>,
    pub main_subject: Option<String>,
    pub contains_pets: bool,
    pub contains_vehicle: bool,
    pub contains_landmarks: bool,
    pub contains_people: bool,
    pub contains_animals: bool,
    pub is_indoor: bool,
    pub is_food_or_drink: bool,
    pub is_event: bool,
    pub is_document: bool,
    pub is_landscape: bool,
    pub is_cityscape: bool,
    pub is_activity: bool,
    pub setting: Option<String>,
    pub pet_type: Option<String>,
    pub animal_type: Option<String>,
    pub food_or_drink_type: Option<String>,
    pub vehicle_type: Option<String>,
    pub event_type: Option<String>,
    pub landmark_name: Option<String>,
    pub document_type: Option<String>,
    pub people_count: Option<i32>,
    pub people_mood: Option<String>,
    pub photo_type: Option<String>,
    pub activity_description: Option<String>,
}

/// Corresponds to the 'time_details' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeDetails {
    pub media_item_id: String,
    pub timezone_name: Option<String>,
    pub timezone_offset_seconds: Option<i32>,
    pub source: Option<String>,
    pub source_details: Option<String>,
    pub source_confidence: Option<String>,
}

/// Corresponds to the 'weather' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub media_item_id: String,
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
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub media_item_id: String,
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

/// Corresponds to the 'capture_details' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDetails {
    pub media_item_id: String,
    pub iso: Option<i32>,
    pub exposure_time: Option<f32>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

/// Corresponds to the 'panorama' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Panorama {
    pub media_item_id: String,
    pub is_photosphere: bool,
    pub projection_type: Option<String>,
    pub horizontal_fov_deg: Option<f32>,
    pub vertical_fov_deg: Option<f32>,
    pub center_yaw_deg: Option<f32>,
    pub center_pitch_deg: Option<f32>,
}

/// A composite struct representing data from the 'ocr_data' table, with its associated OCR boxes nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OcrData {
    pub id: i64,
    pub visual_analysis_id: i64,
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    pub boxes: Vec<OcrBox>,
}

/// A composite struct representing a 'visual_analysis' run and all its associated nested data.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VisualAnalysis {
    pub id: i64,
    pub media_item_id: String,
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Vec<f32>)]
    pub embedding: Option<Vec<f32>>,
    pub ocr_data: Vec<OcrData>,
    pub faces: Vec<Face>,
    pub detected_objects: Vec<DetectedObject>,
    pub quality: Option<QualityData>,
    pub colors: Option<ColorData>,
    pub caption: Option<CaptionData>,
}

/// A composite struct representing data from the 'gps' table, with its associated 'location' data nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Gps {
    pub media_item_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub image_direction: Option<f64>,
    pub location: Option<Location>,
}

/// The root struct representing a 'media_item' and all its available, nested information.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
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

#[derive(sqlx::FromRow, Debug)]
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
    // THIS LINE IS CHANGED
    pub visual_analyses: Option<sqlx::types::Json<Vec<VisualAnalysis>>>,
    pub gps: Option<sqlx::types::Json<Gps>>,
    pub time_details: Option<sqlx::types::Json<TimeDetails>>,
    pub weather: Option<sqlx::types::Json<Weather>>,
    pub details: Option<sqlx::types::Json<Details>>,
    pub capture_details: Option<sqlx::types::Json<CaptureDetails>>,
    pub panorama: Option<sqlx::types::Json<Panorama>>,
}

impl From<FullMediaItemRow> for FullMediaItem {
    fn from(row: FullMediaItemRow) -> Self {
        Self {
            id: row.id,
            hash: row.hash,
            relative_path: row.relative_path,
            created_at: row.created_at,
            updated_at: row.updated_at,
            width: row.width,
            height: row.height,
            is_video: row.is_video,
            duration_ms: row.duration_ms,
            taken_at_local: row.taken_at_local,
            taken_at_utc: row.taken_at_utc,
            use_panorama_viewer: row.use_panorama_viewer,
            // THIS LINE IS CHANGED to safely handle the Option
            visual_analyses: row.visual_analyses.map_or(Vec::new(), |j| j.0),
            gps: row.gps.map(|j| j.0),
            time_details: row.time_details.map(|j| j.0),
            weather: row.weather.map(|j| j.0),
            details: row.details.map(|j| j.0),
            capture_details: row.capture_details.map(|j| j.0),
            panorama: row.panorama.map(|j| j.0),
        }
    }
}
