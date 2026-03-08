use face_id::analyzer::FaceAnalysis;
use face_id::gender_age::Gender;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct FaceEmbedding {
    pub id: i64,
    pub media_item_id: String,
    pub embedding: Vector,
}

/// Corresponds to the 'face' table.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Face {
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: String,
}

/// Corresponds to the 'face' table.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateFace {
    pub embedding: Vector,
    pub position_x: f32,
    pub position_y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub age: i32,
    pub sex: String,
}

impl From<FaceAnalysis> for CreateFace {
    fn from(face_box: FaceAnalysis) -> Self {
        let emb = face_box.embedding.unwrap();
        Self {
            embedding: emb.into(),
            position_x: face_box.detection.bbox.x1,
            position_y: face_box.detection.bbox.y1,
            width: face_box.detection.bbox.width(),
            height: face_box.detection.bbox.height(),
            confidence: face_box.detection.score,
            age: face_box.gender_age.clone().unwrap().age as i32,
            sex: if face_box.gender_age.unwrap().gender == Gender::Male {
                "Male".to_owned()
            } else {
                "Female".to_owned()
            },
        }
    }
}
