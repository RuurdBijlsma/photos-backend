pub use super::_entities::face_boxes::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::{FaceBox, FaceSex};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type FaceBoxes = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn create_face_boxes_from_analysis<C>(
        db: &C,
        face_boxes: &Vec<FaceBox>,
        visual_feature_id: i32,
    ) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        for face in face_boxes {
            let face_box_active = ActiveModel {
                position: Set(face.position.to_vec()),
                width: Set(face.width),
                height: Set(face.height),
                confidence: Set(face.confidence),
                age: Set(face.age),
                sex: Set(match face.sex {
                    FaceSex::Male => "M".to_string(),
                    FaceSex::Female => "F".to_string(),
                }),
                mouth_left: Set(face.mouth_left.to_vec()),
                mouth_right: Set(face.mouth_right.to_vec()),
                nose_tip: Set(face.nose_tip.to_vec()),
                eye_left: Set(face.eye_left.to_vec()),
                eye_right: Set(face.eye_right.to_vec()),
                embedding: Set(face.embedding.clone()),
                visual_feature_id: Set(visual_feature_id),
                ..Default::default()
            };
            face_box_active.insert(db).await?;
        }
        Ok(())
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
