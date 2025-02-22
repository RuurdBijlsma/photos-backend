use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "tags",
            &[
                ("use_panorama_viewer", ColType::Boolean),
                ("is_photosphere", ColType::Boolean),
                ("projection_type", ColType::BooleanNull),
                ("is_motion_photo", ColType::Boolean),
                ("motion_photo_presentation_timestamp", ColType::IntegerNull),
                ("is_night_sight", ColType::Boolean),
                ("is_hdr", ColType::Boolean),
                ("is_burst", ColType::Boolean),
                ("burst_id", ColType::StringNull),
                ("is_timelapse", ColType::Boolean),
                ("is_slowmotion", ColType::Boolean),
                ("is_video", ColType::Boolean),
                ("capture_fps", ColType::FloatNull),
                ("video_fps", ColType::FloatNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "tags").await
    }
}
