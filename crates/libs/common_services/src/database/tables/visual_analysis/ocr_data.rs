use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::database::visual_analysis::ocr_box::OcrBox;

/// A composite struct representing data from the '`ocr_data`' table, with its associated OCR boxes nested inside.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct OcrData {
    pub has_legible_text: bool,
    pub ocr_text: Option<String>,
    pub boxes: Vec<OcrBox>,
}