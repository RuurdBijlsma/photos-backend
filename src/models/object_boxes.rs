pub use super::_entities::object_boxes::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::ObjectBox;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type ObjectBoxes = Entity;

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
    pub async fn create_from_analysis<C>(
        db: &C,
        object_boxes: &Vec<ObjectBox>,
        visual_feature_id: i32,
    ) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        for object in object_boxes {
            let object_box_active = ActiveModel {
                position: Set(object.position.to_vec()),
                width: Set(object.width),
                height: Set(object.height),
                label: Set(object.label.clone()),
                confidence: Set(object.confidence),
                visual_feature_id: Set(visual_feature_id),
                ..Default::default()
            };
            object_box_active.insert(db).await?;
        }
        Ok(())
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
