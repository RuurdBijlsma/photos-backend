use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OCRData {
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    pub ocr_boxes: Option<Vec<OCRBox>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OCRBox {
    pub text: String,
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Variant {
    Monochrome,
    Neutral,
    TonalSpot,
    Vibrant,
    Expressive,
    Fidelity,
    Content,
    Rainbow,
    FruitSalad,
}

impl Variant {
    /// Converts the enum variant to its uppercase string representation.
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Monochrome => "MONOCHROME",
            Self::Neutral => "NEUTRAL",
            Self::TonalSpot => "TONAL_SPOT",
            Self::Vibrant => "VIBRANT",
            Self::Expressive => "EXPRESSIVE",
            Self::Fidelity => "FIDELITY",
            Self::Content => "CONTENT",
            Self::Rainbow => "RAINBOW",
            Self::FruitSalad => "FRUIT_SALAD",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FaceBox {
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: String,
    pub mouth_left: (f32, f32),
    pub mouth_right: (f32, f32),
    pub nose_tip: (f32, f32),
    pub eye_left: (f32, f32),
    pub eye_right: (f32, f32),
    pub embedding: Vec<f32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ObjectBox {
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct MeasuredQualityData {
    pub measured_sharpness: f32,
    pub measured_noise: i32,
    pub measured_brightness: f32,
    pub measured_contrast: f32,
    pub measured_clipping: f32,
    pub measured_dynamic_range: f32,
    pub quality_score: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ColorData {
    pub themes: Vec<Value>,
    pub prominent_colors: Vec<String>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
    pub histogram: ColorHistogram,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ColorHistogram {
    pub bins: i32,
    pub channels: RGBChannels,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct RGBChannels {
    pub red: Vec<i32>,
    pub green: Vec<i32>,
    pub blue: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct CaptionData {
    pub default_caption: String,
    pub main_subject: String,
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
    pub setting: String,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct VisualImageData {
    pub color_data: ColorData,
    pub caption_data: CaptionData,
    pub embedding: Vec<f32>,
    pub faces: Vec<FaceBox>,
    pub objects: Vec<ObjectBox>,
    pub ocr: OCRData,
}
