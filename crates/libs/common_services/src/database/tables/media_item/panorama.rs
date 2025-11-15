use media_analyzer::PanoInfo;
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

/// Converts from the analysis result's `PanoInfo` to the database model `Panorama`.
impl From<PanoInfo> for Panorama {
    fn from(pano_info: PanoInfo) -> Self {
        Self {
            is_photosphere: pano_info.is_photosphere,
            projection_type: pano_info.projection_type,
            horizontal_fov_deg: pano_info
                .view_info
                .as_ref()
                .map(|vi| vi.horizontal_fov_deg as f32),
            vertical_fov_deg: pano_info
                .view_info
                .as_ref()
                .map(|vi| vi.vertical_fov_deg as f32),
            center_yaw_deg: pano_info
                .view_info
                .as_ref()
                .map(|vi| vi.center_yaw_deg as f32),
            center_pitch_deg: pano_info
                .view_info
                .map(|vi| vi.center_pitch_deg as f32),
        }
    }
}