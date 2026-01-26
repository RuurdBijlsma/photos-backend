use common_types::ml_analysis;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Corresponds to the '`judged_quality`' table.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QualityScore {
    pub exposure: u8,
    pub contrast: u8,
    pub sharpness: u8,
    pub color_accuracy: u8,
    pub composition: u8,
    pub subject_clarity: u8,
    pub visual_impact: u8,
    pub creativity: u8,
    pub color_harmony: u8,
    pub storytelling: u8,
    pub style_suitability: u8,
    pub weighted_score: f64,

    pub measured_blurriness: f64,
    pub measured_noisiness: f64,
    pub measured_exposure: f64,
    pub measured_weighted_score: f64,
}

impl From<ml_analysis::CombinedQuality> for QualityScore {
    fn from(combined_quality: ml_analysis::CombinedQuality) -> Self {
        let measured = combined_quality.measured;
        combined_quality.judged.map_or(Self {
                exposure: 0,
                contrast: 0,
                sharpness: 0,
                color_accuracy: 0,
                composition: 0,
                subject_clarity: 0,
                visual_impact: 0,
                creativity: 0,
                color_harmony: 0,
                storytelling: 0,
                style_suitability: 0,
                weighted_score: 0.,

                measured_noisiness: measured.noisiness,
                measured_exposure: measured.exposure,
                measured_blurriness: measured.blurriness,
                measured_weighted_score: measured.weighted_score,
            }, |judged| Self {
                exposure: judged.exposure,
                contrast: judged.contrast,
                sharpness: judged.sharpness,
                color_accuracy: judged.color_accuracy,
                composition: judged.composition,
                subject_clarity: judged.subject_clarity,
                visual_impact: judged.visual_impact,
                creativity: judged.creativity,
                color_harmony: judged.color_harmony,
                storytelling: judged.storytelling,
                style_suitability: judged.style_suitability,
                weighted_score: f64::from(judged.weighted_score()),

                measured_noisiness: measured.noisiness,
                measured_exposure: measured.exposure,
                measured_blurriness: measured.blurriness,
                measured_weighted_score: measured.weighted_score,
            })
    }
}
