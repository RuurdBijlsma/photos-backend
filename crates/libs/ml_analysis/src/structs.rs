use pyo3::{IntoPyObject, IntoPyObjectExt, Python};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use pyo3::prelude::*;



#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    Assistant,
    User,
}

impl<'py> IntoPyObject<'py> for ChatRole {
    type Target = PyAny; // the Python type
    type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
    type Error = std::convert::Infallible; // the conversion error type, has to be convertible to `PyErr`

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::User => {
                let result = "user".into_bound_py_any(py).unwrap();
                Ok(result)
            },
            Self::Assistant => {
                let result = "assistant".into_bound_py_any(py).unwrap();
                Ok(result)
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, IntoPyObject)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, FromRow)]
pub struct OCRData {
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    #[sqlx(skip)]
    pub ocr_boxes: Option<Vec<OCRBox>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, FromRow)]
pub struct OCRBox {
    pub text: String,
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, FromRow)]
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, FromRow)]
pub struct ObjectBox {
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, FromRow)]
pub struct QualityData {
    pub blurriness: f64,
    pub noisiness: f64,
    pub exposure: f64,
    pub quality_score: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, FromRow)]
pub struct ColorData {
    #[sqlx(json)]
    pub themes: Vec<Value>,
    pub prominent_colors: Vec<String>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
    #[sqlx(json)]
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, FromRow)]
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

// This top-level struct is assembled manually, so it does not need FromRow
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct VisualImageData {
    pub color_data: ColorData,
    pub quality_data: QualityData,
    pub caption_data: CaptionData,
    pub embedding: Vec<f32>,
    pub faces: Vec<FaceBox>,
    pub objects: Vec<ObjectBox>,
    pub ocr: OCRData,
}
