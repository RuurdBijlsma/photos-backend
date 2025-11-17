use crate::database::visual_analysis::ocr_box::OCRBox;
use common_types::ml_analysis::PyOCRData;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// A composite struct representing data from the '`ocr_data`' table, with its associated OCR boxes nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct OCRData {
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    pub boxes: Vec<OCRBox>,
}
impl From<PyOCRData> for OCRData {
    fn from(ocr_data: PyOCRData) -> Self {
        Self {
            has_legible_text: ocr_data.has_legible_text,
            ocr_text: ocr_data.ocr_text,
            boxes: ocr_data
                .ocr_boxes
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
