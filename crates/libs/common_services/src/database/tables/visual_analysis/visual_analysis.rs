use crate::database::visual_analysis::caption_data::CaptionData;
use crate::database::visual_analysis::color_data::ColorData;
use crate::database::visual_analysis::detect_object::DetectedObject;
use crate::database::visual_analysis::face::{CreateFace, Face};
use crate::database::visual_analysis::ocr_data::OCRData;
use crate::database::visual_analysis::quality_data::QualityData;
use chrono::{DateTime, Utc};
use common_types::ml_analysis::PyVisualAnalysis;
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
    pub ocr_data: OCRData,
    pub faces: Vec<CreateFace>,
    pub detected_objects: Vec<DetectedObject>,
    pub quality: QualityData,
    pub colors: ColorData,
    pub caption: CaptionData,
}

impl From<PyVisualAnalysis> for CreateVisualAnalysis {
    fn from(data: PyVisualAnalysis) -> Self {
        Self {
            embedding: data.embedding.into(),
            percentage: data.percentage,
            ocr_data: data.ocr.into(),
            faces: data.faces.into_iter().map(Into::into).collect(),
            detected_objects: data.objects.into_iter().map(Into::into).collect(),
            quality: data.quality_data.into(),
            colors: data.color_data.into(),
            caption: data.caption_data.into(),
        }
    }
}

/// A composite struct representing a '`visual_analysis`' run and all its associated nested data.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ReadVisualAnalysis {
    pub created_at: DateTime<Utc>,
    pub percentage: i32,
    pub ocr_data: OCRData,
    pub faces: Vec<Face>,
    pub detected_objects: Vec<DetectedObject>,
    pub quality: QualityData,
    pub colors: ColorData,
    pub caption: CaptionData,
}
