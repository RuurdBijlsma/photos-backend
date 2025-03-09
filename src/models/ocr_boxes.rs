pub use super::_entities::ocr_boxes::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::OCRBox;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type OcrBoxes = Entity;

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
    /// Create ocr boxes based on `MediaAnalyzerOutput`, and store it in db.
    ///
    /// # Errors
    /// If an INSERT fails.
    pub async fn create_from_analysis<C>(
        db: &C,
        ocr_boxes: &Vec<OCRBox>,
        visual_feature_id: i32,
    ) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        for ocr_box in ocr_boxes {
            let ocr_box_active = Self {
                position: Set(ocr_box.position.to_vec()),
                width: Set(ocr_box.width),
                height: Set(ocr_box.height),
                confidence: Set(ocr_box.confidence),
                text: Set(ocr_box.text.clone()),
                visual_feature_id: Set(visual_feature_id),
                ..Default::default()
            };
            ocr_box_active.insert(db).await?;
        }
        Ok(())
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
