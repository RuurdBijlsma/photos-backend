pub use super::_entities::tags::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::TagData;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type Tags = Entity;

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
        tags: TagData,
        image_id: String,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let tags = ActiveModel {
            use_panorama_viewer: Set(tags.use_panorama_viewer),
            is_photosphere: Set(tags.is_photosphere),
            projection_type: Set(tags.projection_type),
            is_motion_photo: Set(tags.is_motion_photo),
            motion_photo_presentation_timestamp: Set(tags.motion_photo_presentation_timestamp),
            is_night_sight: Set(tags.is_night_sight),
            is_hdr: Set(tags.is_hdr),
            is_burst: Set(tags.is_burst),
            burst_id: Set(tags.burst_id),
            is_timelapse: Set(tags.is_timelapse),
            is_slowmotion: Set(tags.is_slowmotion),
            is_video: Set(tags.is_video),
            capture_fps: Set(tags.capture_fps),
            video_fps: Set(tags.video_fps),
            image_id: Set(image_id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(tags)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
