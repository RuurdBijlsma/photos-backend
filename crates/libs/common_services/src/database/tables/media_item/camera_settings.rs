use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Corresponds to the '`capture_details`' table.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CameraSettings {
    pub iso: Option<i32>,
    pub exposure_time: Option<f32>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}

/// Converts from the analysis result's `SourceCaptureDetails` to the database model `CaptureDetails`.
impl From<media_analyzer::CameraSettings> for CameraSettings {
    fn from(details: media_analyzer::CameraSettings) -> Self {
        Self {
            iso: details.iso.map(|iso| iso as i32),
            exposure_time: details.exposure_time.map(|et| et as f32),
            aperture: details.aperture.map(|a| a as f32),
            focal_length: details.focal_length.map(|fl| fl as f32),
            camera_make: details.camera_make,
            camera_model: details.camera_model,
        }
    }
}
