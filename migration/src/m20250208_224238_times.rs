use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "times",
            &[
                ("datetime_local", ColType::Timestamp),
                ("datetime_utc", ColType::TimestampNull),
                ("datetime_source", ColType::String),
                ("timezone_name", ColType::StringNull),
                ("timezone_offset", ColType::StringNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "times").await
    }
}
