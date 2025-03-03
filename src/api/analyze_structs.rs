use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingRequest {
    pub relative_path: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessingJob {
    job_id: String,
    pub(crate) result: Option<MediaAnalyzerOutput>,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Deserialize)]
pub struct MediaAnalyzerOutput {
    pub image_data: ImageDataOutput,
    pub frame_data: Vec<FrameDataOutput>,
}

#[derive(Debug, Deserialize)]
pub struct ImageDataOutput {
    pub path: String,
    pub exif: ExifData,
    pub data_url: String,
    pub gps: Option<GPSData>,
    pub time: TimeData,
    pub weather: Option<WeatherData>,
    pub tags: TagData,
}

#[derive(Debug, Deserialize)]
pub struct FrameDataOutput {
    pub ocr: OCRData,
    pub embedding: Vec<f32>,
    pub faces: Vec<FaceBox>,
    pub summary: Option<String>,
    pub caption: Option<String>,
    pub objects: Vec<ObjectBox>,
    pub classification: ClassificationData,
    pub measured_quality: MeasuredQualityData,
    pub color: ColorData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeData {
    pub datetime_local: String, // ISO 8601 date-time
    pub datetime_source: String,
    pub timezone_name: Option<String>,
    pub timezone_offset: Option<String>, // Duration format (e.g., "PT1H" -> period of 1 hour)
    pub datetime_utc: Option<String>,    // ISO 8601 date-time
}
#[derive(Debug, Deserialize)]
pub struct ExifData {
    pub width: i32,
    pub height: i32,
    pub duration: Option<f32>,
    pub size_bytes: i64,
    pub format: String,
    pub exif_tool: Value,
    pub file: Value,
    pub composite: Value,
    pub exif: Option<Value>,
    pub xmp: Option<Value>,
    pub mpf: Option<Value>,
    pub jfif: Option<Value>,
    pub icc_profile: Option<Value>,
    pub gif: Option<Value>,
    pub png: Option<Value>,
    pub quicktime: Option<Value>,
    pub matroska: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct GPSData {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: Option<f32>,
    pub location: GeoLocation,
}

#[derive(Debug, Deserialize)]
pub struct GeoLocation {
    pub country: String,
    pub city: String,
    pub province: Option<String>,
    pub place_latitude: f32,
    pub place_longitude: f32,
}

// WeatherData
#[derive(Debug, Deserialize, Serialize)]
pub struct WeatherData {
    pub weather_recorded_at: Option<String>, // ISO 8601 date-time
    pub weather_temperature: Option<f32>,
    pub weather_dewpoint: Option<f32>,
    pub weather_relative_humidity: Option<f32>,
    pub weather_precipitation: Option<f32>,
    pub weather_wind_gust: Option<f32>,
    pub weather_pressure: Option<f32>,
    pub weather_sun_hours: Option<f32>,
    pub weather_condition: Option<i32>,
}

// TagData
#[derive(Debug, Deserialize, Serialize)]
pub struct TagData {
    pub use_panorama_viewer: bool,
    pub is_photosphere: bool,
    pub projection_type: Option<String>,
    pub is_motion_photo: bool,
    pub motion_photo_presentation_timestamp: Option<i64>,
    pub is_night_sight: bool,
    pub is_hdr: bool,
    pub is_burst: bool,
    pub burst_id: Option<String>,
    pub is_timelapse: bool,
    pub is_slowmotion: bool,
    pub is_video: bool,
    pub capture_fps: Option<f32>,
    pub video_fps: Option<f32>,
}

// OCRData
#[derive(Debug, Deserialize, Serialize)]
pub struct OCRData {
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    pub document_summary: Option<String>,
    pub ocr_boxes: Vec<OCRBox>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OCRBox {
    pub position: [f32; 2], // [x, y]
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub text: String,
}

// FaceBox
#[derive(Debug, Deserialize, Serialize)]
pub struct FaceBox {
    pub position: [f32; 2], // [x, y]
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: FaceSex,
    pub mouth_left: [f32; 2],
    pub mouth_right: [f32; 2],
    pub nose_tip: [f32; 2],
    pub eye_left: [f32; 2],
    pub eye_right: [f32; 2],
    pub embedding: Vec<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FaceSex {
    #[serde(rename = "M")]
    Male,
    #[serde(rename = "F")]
    Female,
}

// ObjectBox
#[derive(Debug, Deserialize, Serialize)]
pub struct ObjectBox {
    pub position: [f32; 2], // [x, y]
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub label: String,
}

// ClassificationData
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassificationData {
    pub scene_type: String,
    pub people_type: Option<String>,
    pub animal_type: Option<String>,
    pub document_type: Option<String>,
    pub object_type: Option<String>,
    pub activity_type: Option<String>,
    pub event_type: Option<String>,
    pub weather_condition: Option<i32>,
    pub is_outside: bool,
    pub is_landscape: bool,
    pub is_cityscape: bool,
    pub is_travel: bool,
}

// MeasuredQualityData
#[derive(Debug, Deserialize, Serialize)]
pub struct MeasuredQualityData {
    pub measured_sharpness: f32,
    pub measured_noise: i32,
    pub measured_brightness: f32,
    pub measured_contrast: f32,
    pub measured_clipping: f32,
    pub measured_dynamic_range: f32,
    pub quality_score: f32,
}

// ColorData
#[derive(Debug, Deserialize, Serialize)]
pub struct ColorData {
    pub themes: Vec<Value>,
    pub prominent_colors: Vec<String>,
    pub average_hue: f32,
    pub average_saturation: f32,
    pub average_lightness: f32,
}
