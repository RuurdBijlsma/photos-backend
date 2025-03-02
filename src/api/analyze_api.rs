use crate::common::settings::Settings;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{error, info, warn};

// -- Data structures for Processing API --
#[derive(Debug, Serialize, Deserialize)]
struct ProcessingRequest {
    relative_path: String,
}

#[derive(Debug, Deserialize)]
struct ProcessingJob {
    job_id: String,
    result: Option<MediaAnalyzerOutput>,
    #[serde(default)]
    done: bool,
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
    ocr: Option<OCRData>,
    embedding: Option<Vec<f64>>,
    faces: Option<Vec<FaceBox>>,
    summary: Option<String>,
    caption: Option<String>,
    objects: Option<Vec<ObjectBox>>,
    classification: Option<ClassificationData>,
    measured_quality: Option<MeasuredQualityData>,
    color: Option<ColorData>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TimeData {
    datetime_local: String, // ISO 8601 date-time
    datetime_source: String,
    timezone_name: Option<String>,
    timezone_offset: Option<String>, // Duration format (e.g., "+02:00")
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
    latitude: Option<f64>,
    longitude: Option<f64>,
    altitude: Option<f64>,
    location: Option<GeoLocation>,
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

// -- API Client --
struct ProcessClient {
    client: Client,
    base_url: String,
}

impl ProcessClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .read_timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
        }
    }

    pub async fn submit_job(
        &self,
        request: &ProcessingRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/process", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {status}: {text}").into())
            }
        }
    }

    pub async fn check_status(
        &self,
        job_id: &str,
    ) -> Result<ProcessingJob, Box<dyn std::error::Error>> {
        let url = format!("{}/process/{}", self.base_url, job_id);
        let response = self.client.get(&url).send().await?;
        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {status}: {text}").into())
            }
        }
    }

    pub async fn delete_job(&self, job_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/process/{}", self.base_url, job_id);
        let response = self.client.delete(&url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {status}: {text}").into())
            }
        }
    }
}

pub async fn process_media(
    relative_path: String,
    settings: &Settings,
) -> Result<MediaAnalyzerOutput, loco_rs::Error> {
    const MAX_RETRIES: u64 = 3;
    const RETRY_DELAY: u64 = 1;
    let client = ProcessClient::new(&settings.processing_api_url);
    let request = ProcessingRequest { relative_path };

    // Submit job
    let job_id = client
        .submit_job(&request)
        .await
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
    info!("Submitted processing job: {}", job_id);

    // Polling parameters
    let delay = 5;
    let timeout = 300; // 5 minutes
    let mut attempts = 0;

    loop {
        tokio::time::sleep(Duration::from_secs(delay)).await;

        // Check status with retry logic
        let mut retries: u64 = 0;
        let status = loop {
            match client
                .check_status(&job_id)
                .await
                .map_err(|e| loco_rs::Error::Message(e.to_string()))
            {
                Ok(status) => break status,
                Err(e) => {
                    if retries >= MAX_RETRIES {
                        return Err(e);
                    }
                    retries += 1;
                    let sleep_duration = Duration::from_secs(RETRY_DELAY * retries);
                    tokio::time::sleep(sleep_duration).await;
                    info!(
                        "Retrying check_status for job {} (attempt {}/{})",
                        job_id, retries, MAX_RETRIES
                    );
                }
            }
        };

        info!("Job {} status: done={}", job_id, status.done);

        if status.done {
            return if let Some(result) = status.result {
                // Clean up the job
                client
                    .delete_job(&job_id)
                    .await
                    .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
                info!("Deleted completed job {}", job_id);
                Ok(result)
            } else {
                error!("Job {} marked done but no result available", job_id);
                Err(loco_rs::Error::Message(
                    "Processing completed but no result available".to_string(),
                ))
            };
        }

        attempts += 1;
        if attempts * delay >= timeout {
            error!("Job {} timed out after {} attempts", job_id, attempts);
            return Err(loco_rs::Error::Message(
                "Processing request timed out".to_string(),
            ));
        }
    }
}
