use crate::database::visual_analysis::caption_data::ClassificationData;
use crate::database::visual_analysis::color_data::ColorData;
use crate::database::visual_analysis::face::{CreateFace, Face};
use crate::database::visual_analysis::quality::QualityScore;
use chrono::{DateTime, Utc};
use common_types::ml_analysis::MLVisualAnalysis;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a single photo's embedding data fetched for clustering.
#[derive(Debug, Clone)]
pub struct MediaEmbedding {
    pub media_item_id: String,
    pub embedding: Vector,
}

/// A composite struct representing a '`visual_analysis`' run and all its associated nested data.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateVisualAnalysis {
    pub embedding: Vector,
    pub percentage: i32,
    pub faces: Vec<CreateFace>,
    pub quality: QualityScore,
    pub colors: ColorData,
    pub classification: ClassificationData,
}

impl From<MLVisualAnalysis> for CreateVisualAnalysis {
    fn from(data: MLVisualAnalysis) -> Self {
        Self {
            embedding: data.embedding.into(),
            percentage: data.percentage,
            faces: data.faces.into_iter().map(Into::into).collect(),
            quality: data.quality.into(),
            colors: data.color_data.into(),
            classification: data.llm_classification.into(),
        }
    }
}

/// A composite struct representing a '`visual_analysis`' run and all its associated nested data.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ReadVisualAnalysis {
    pub created_at: DateTime<Utc>,
    pub percentage: i32,
    pub faces: Vec<Face>,
    pub quality: QualityScore,
    pub colors: ColorData,
    pub classification: ClassificationData,
}
