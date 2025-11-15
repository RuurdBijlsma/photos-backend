use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use common_types::ml_analysis_types::ObjectBox;

/// Corresponds to the '`detected_object`' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct DetectedObject {
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub label: String,
}

impl From<ObjectBox> for DetectedObject {
    fn from(object_box: ObjectBox) -> Self {
        Self {
            position_x: object_box.position.0,
            position_y: object_box.position.1,
            width: object_box.width,
            height: object_box.height,
            confidence: object_box.confidence,
            label: object_box.label,
        }
    }
}