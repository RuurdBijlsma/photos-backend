use common_types::ml_analysis_types::PyOCRBox;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Corresponds to the '`ocr_box`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct OCRBox {
    pub text: String,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}
/// Converts from the analysis result's `SourceOCRBox` to the database model `OcrBox`.
impl From<PyOCRBox> for OCRBox {
    fn from(box_data: PyOCRBox) -> Self {
        Self {
            text: box_data.text,
            position_x: box_data.position.0,
            position_y: box_data.position.1,
            width: box_data.width,
            height: box_data.height,
            confidence: box_data.confidence,
        }
    }
}
