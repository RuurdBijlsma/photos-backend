use common_types::ml_analysis_types::FaceBox;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, FromRow, Clone)]
pub struct FaceEmbedding {
    pub id: i64,
    pub media_item_id: String,
    pub embedding: Vector,
}

/// Corresponds to the 'face' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Face {
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: String,
    pub mouth_left_x: f32,
    pub mouth_left_y: f32,
    pub mouth_right_x: f32,
    pub mouth_right_y: f32,
    pub nose_tip_x: f32,
    pub nose_tip_y: f32,
    pub eye_left_x: f32,
    pub eye_left_y: f32,
    pub eye_right_x: f32,
    pub eye_right_y: f32,
}

/// Corresponds to the 'face' table.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct CreateFace {
    pub embedding: Vector,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: String,
    pub mouth_left_x: f32,
    pub mouth_left_y: f32,
    pub mouth_right_x: f32,
    pub mouth_right_y: f32,
    pub nose_tip_x: f32,
    pub nose_tip_y: f32,
    pub eye_left_x: f32,
    pub eye_left_y: f32,
    pub eye_right_x: f32,
    pub eye_right_y: f32,
}

impl From<FaceBox> for CreateFace {
    fn from(face_box: FaceBox) -> Self {
        Self {
            embedding: face_box.embedding.into(),
            position_x: face_box.position.0,
            position_y: face_box.position.1,
            width: face_box.width,
            height: face_box.height,
            confidence: face_box.confidence,
            age: face_box.age,
            sex: face_box.sex,
            mouth_left_x: face_box.mouth_left.0,
            mouth_left_y: face_box.mouth_left.1,
            mouth_right_x: face_box.mouth_right.0,
            mouth_right_y: face_box.mouth_right.1,
            nose_tip_x: face_box.nose_tip.0,
            nose_tip_y: face_box.nose_tip.1,
            eye_left_x: face_box.eye_left.0,
            eye_left_y: face_box.eye_left.1,
            eye_right_x: face_box.eye_right.0,
            eye_right_y: face_box.eye_right.1,
        }
    }
}
