use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`ocr_box`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct OcrBox {
    pub text: String,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

/// Corresponds to the 'face' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Face {
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
}

/// Corresponds to the '`detected_object`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct DetectedObject {
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub label: String,
}

/// Corresponds to the '`quality_data`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct QualityData {
    pub blurriness: f64,
    pub noisiness: f64,
    pub exposure: f64,
    pub quality_score: f64,
}

/// Corresponds to the '`color_data`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct ColorData {
    pub themes: Vec<Value>,
    pub prominent_colors: Option<Vec<String>>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
    pub histogram: Option<Value>,
}

/// Corresponds to the '`caption_data`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct CaptionData {
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

/// A composite struct representing data from the '`ocr_data`' table, with its associated OCR boxes nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct OcrData {
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    pub boxes: Vec<OcrBox>,
}

/// A composite struct representing a '`visual_analysis`' run and all its associated nested data.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct VisualAnalysis {
    pub created_at: DateTime<Utc>,
    pub ocr_data: Vec<OcrData>,
    pub faces: Vec<Face>,
    pub detected_objects: Vec<DetectedObject>,
    pub quality: Option<QualityData>,
    pub colors: Option<ColorData>,
    pub caption: Option<CaptionData>,
}