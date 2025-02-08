use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {

        create_table(m, "weather",
            &[
            ("weather_recorded_at", ColType::TimestampNull),
            ("weather_temperature", ColType::FloatNull),
            ("weather_dewpoint", ColType::FloatNull),
            ("weather_relative_humidity", ColType::FloatNull),
            ("weather_precipitation", ColType::FloatNull),
            ("weather_wind_gust", ColType::FloatNull),
            ("weather_pressure", ColType::FloatNull),
            ("weather_sun_hours", ColType::FloatNull),
            ("weather_condition", ColType::StringNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "weather").await
    }
}
