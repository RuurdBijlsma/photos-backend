pub use super::_entities::metadata::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::ExifData;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type Metadata = Entity;

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
        metadata: ExifData,
        image_id: String,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let metadata = ActiveModel {
            exif_tool: Set(metadata.exif_tool),
            file: Set(metadata.file),
            composite: Set(metadata.composite),
            exif: Set(metadata.exif),
            xmp: Set(metadata.xmp),
            mpf: Set(metadata.mpf),
            jfif: Set(metadata.jfif),
            icc_profile: Set(metadata.icc_profile),
            gif: Set(metadata.gif),
            png: Set(metadata.png),
            quicktime: Set(metadata.quicktime),
            matroska: Set(metadata.matroska),
            image_id: Set(image_id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(metadata)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
