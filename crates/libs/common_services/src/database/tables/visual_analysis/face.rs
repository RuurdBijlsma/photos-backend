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
