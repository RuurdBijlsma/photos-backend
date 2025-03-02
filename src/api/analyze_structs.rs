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
    image_data: ImageDataOutput,
    frame_data: Vec<FrameDataOutput>,
}

#[derive(Debug, Deserialize)]
struct ImageDataOutput {
    path: String,
    exif: ExifData,
    data_url: String,
    gps: Option<GPSData>,
    time: TimeData,
    weather: Option<WeatherData>,
    tags: TagData,
}

#[derive(Debug, Deserialize)]
struct FrameDataOutput {
    ocr: OCRData,
    embedding: Vec<f64>,
    faces: Vec<FaceBox>,
    summary: Option<String>,
    caption: Option<String>,
    objects: Vec<ObjectBox>,
    classification: ClassificationData,
    measured_quality: MeasuredQualityData,
    color: ColorData,
}

#[derive(Debug, Deserialize, Serialize)]
struct TimeData {
    datetime_local: String, // ISO 8601 date-time
    datetime_source: String,
    timezone_name: Option<String>,
    timezone_offset: Option<String>, // Duration format (e.g., "PT1H" -> period of 1 hour)
    datetime_utc: Option<String>,    // ISO 8601 date-time
}
#[derive(Debug, Deserialize)]
struct ExifData {
    width: i32,
    height: i32,
    duration: Option<f64>,
    size_bytes: i64,
    format: String,
    exif_tool: Value,
    file: Value,
    composite: Value,
    exif: Option<Value>,
    xmp: Option<Value>,
    mpf: Option<Value>,
    jfif: Option<Value>,
    icc_profile: Option<Value>,
    gif: Option<Value>,
    png: Option<Value>,
    quicktime: Option<Value>,
    matroska: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct GPSData {
    latitude: f64,
    longitude: f64,
    altitude: Option<f64>,
    location: GeoLocation,
}

#[derive(Debug, Deserialize)]
struct GeoLocation {
    country: String,
    city: String,
    province: Option<String>,
    place_latitude: f64,
    place_longitude: f64,
}

// WeatherData
#[derive(Debug, Deserialize, Serialize)]
struct WeatherData {
    weather_recorded_at: Option<String>, // ISO 8601 date-time
    weather_temperature: Option<f64>,
    weather_dewpoint: Option<f64>,
    weather_relative_humidity: Option<f64>,
    weather_precipitation: Option<f64>,
    weather_wind_gust: Option<f64>,
    weather_pressure: Option<f64>,
    weather_sun_hours: Option<f64>,
    weather_condition: Option<i32>,
}

// TagData
#[derive(Debug, Deserialize, Serialize)]
struct TagData {
    use_panorama_viewer: bool,
    is_photosphere: bool,
    projection_type: Option<String>,
    is_motion_photo: bool,
    motion_photo_presentation_timestamp: Option<i64>,
    is_night_sight: bool,
    is_hdr: bool,
    is_burst: bool,
    burst_id: Option<String>,
    is_timelapse: bool,
    is_slowmotion: bool,
    is_video: bool,
    capture_fps: Option<f64>,
    video_fps: Option<f64>,
}

// OCRData
#[derive(Debug, Deserialize, Serialize)]
struct OCRData {
    has_legible_text: bool,
    ocr_text: Option<String>,
    document_summary: Option<String>,
    ocr_boxes: Vec<OCRBox>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OCRBox {
    position: [f64; 2], // [x, y]
    width: f64,
    height: f64,
    confidence: f64,
    text: String,
}

// FaceBox
#[derive(Debug, Deserialize, Serialize)]
struct FaceBox {
    position: [f64; 2], // [x, y]
    width: f64,
    height: f64,
    confidence: f64,
    age: i32,
    sex: FaceSex,
    mouth_left: [f64; 2],
    mouth_right: [f64; 2],
    nose_tip: [f64; 2],
    eye_left: [f64; 2],
    eye_right: [f64; 2],
    embedding: Vec<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
enum FaceSex {
    #[serde(rename = "M")]
    Male,
    #[serde(rename = "F")]
    Female,
}

// ObjectBox
#[derive(Debug, Deserialize, Serialize)]
struct ObjectBox {
    position: [f64; 2], // [x, y]
    width: f64,
    height: f64,
    confidence: f64,
    label: String,
}

// ClassificationData
#[derive(Debug, Deserialize, Serialize)]
struct ClassificationData {
    scene_type: String,
    people_type: Option<String>,
    animal_type: Option<String>,
    document_type: Option<String>,
    object_type: Option<String>,
    activity_type: Option<String>,
    event_type: Option<String>,
    weather_condition: Option<i32>,
    is_outside: bool,
    is_landscape: bool,
    is_cityscape: bool,
    is_travel: bool,
}

// MeasuredQualityData
#[derive(Debug, Deserialize, Serialize)]
struct MeasuredQualityData {
    measured_sharpness: f64,
    measured_noise: i32,
    measured_brightness: f64,
    measured_contrast: f64,
    measured_clipping: f64,
    measured_dynamic_range: f64,
    quality_score: f64,
}

// ColorData
#[derive(Debug, Deserialize, Serialize)]
struct ColorData {
    themes: Vec<Value>,
    prominent_colors: Vec<String>,
    average_hue: f64,
    average_saturation: f64,
    average_lightness: f64,
}
