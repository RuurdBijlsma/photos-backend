use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the 'panorama' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Panorama {
    pub is_photosphere: bool,
    pub projection_type: Option<String>,
    pub horizontal_fov_deg: Option<f32>,
    pub vertical_fov_deg: Option<f32>,
    pub center_yaw_deg: Option<f32>,
    pub center_pitch_deg: Option<f32>,
}