use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Corresponds to the '`media_features`' table.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MediaFeatures {
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
    pub exif: Value,
}
