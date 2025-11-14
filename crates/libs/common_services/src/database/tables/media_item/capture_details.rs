use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`capture_details`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct CaptureDetails {
    pub iso: Option<i32>,
    pub exposure_time: Option<f32>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
}
