use common_types::ml_analysis::QualityMeasurement;
use serde::{Deserialize, Serialize};

/// Corresponds to the '`judged_quality`' table.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeasuredQuality {
    pub measured_blurriness: f64,
    pub measured_noisiness: f64,
    pub measured_exposure: f64,
    pub measured_weighted_score: f64,
}

impl From<QualityMeasurement> for MeasuredQuality {
    fn from(measured: QualityMeasurement) -> Self {
        Self {
            measured_noisiness: measured.noisiness,
            measured_exposure: measured.exposure,
            measured_blurriness: measured.blurriness,
            measured_weighted_score: measured.weighted_score,
        }
    }
}
