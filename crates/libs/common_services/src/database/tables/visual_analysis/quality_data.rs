use common_types::ml_analysis;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Corresponds to the '`quality_data`' table.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QualityData {
    pub blurriness: f64,
    pub noisiness: f64,
    pub exposure: f64,
    pub quality_score: f64,
}

impl From<ml_analysis::PyQualityData> for QualityData {
    fn from(quality_data: ml_analysis::PyQualityData) -> Self {
        Self {
            blurriness: quality_data.blurriness,
            noisiness: quality_data.noisiness,
            exposure: quality_data.exposure,
            quality_score: quality_data.quality_score,
        }
    }
}
