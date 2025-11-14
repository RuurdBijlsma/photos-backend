use chrono::{DateTime, Utc};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::database::visual_analysis::caption_data::CaptionData;
use crate::database::visual_analysis::color_data::ColorData;
use crate::database::visual_analysis::detect_object::DetectedObject;
use crate::database::visual_analysis::face::Face;
use crate::database::visual_analysis::ocr_data::OcrData;
use crate::database::visual_analysis::quality_data::QualityData;

/// Represents a single photo's embedding data fetched for clustering.
#[derive(Debug, FromRow, Clone)]
pub struct MediaEmbedding {
    pub media_item_id: String,
    pub embedding: Vector,
}

/// A composite struct representing a '`visual_analysis`' run and all its associated nested data.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct VisualAnalysis {
    pub created_at: DateTime<Utc>,
    pub ocr_data: Vec<OcrData>,
    pub faces: Vec<Face>,
    pub detected_objects: Vec<DetectedObject>,
    pub quality: Option<QualityData>,
    pub colors: Option<ColorData>,
    pub caption: Option<CaptionData>,
}
