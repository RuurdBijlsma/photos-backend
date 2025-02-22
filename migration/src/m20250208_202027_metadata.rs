use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "metadata",
            &[
                ("exif_tool", ColType::JsonBinary),
                ("file", ColType::JsonBinary),
                ("composite", ColType::JsonBinary),
                ("exif", ColType::JsonBinaryNull),
                ("xmp", ColType::JsonBinaryNull),
                ("mpf", ColType::JsonBinaryNull),
                ("jfif", ColType::JsonBinaryNull),
                ("icc_profile", ColType::JsonBinaryNull),
                ("gif", ColType::JsonBinaryNull),
                ("png", ColType::JsonBinaryNull),
                ("quicktime", ColType::JsonBinaryNull),
                ("matroska", ColType::JsonBinaryNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "metadata").await
    }
}
