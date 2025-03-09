use crate::sea_orm::EnumIter;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden, EnumIter)]
pub enum VisualFeatures {
    Table,
    Id,
    UpdatedAt,
    CreatedAt,
    FramePercentage,
    Embedding,
    SceneType,
    PeopleType,
    AnimalType,
    DocumentType,
    ObjectType,
    ActivityType,
    EventType,
    WeatherCondition,
    IsOutside,
    IsLandscape,
    IsCityscape,
    IsTravel,
    HasLegibleText,
    OcrText,
    DocumentSummary,
    MeasuredSharpness,
    MeasuredNoise,
    MeasuredBrightness,
    MeasuredContrast,
    MeasuredClipping,
    MeasuredDynamicRange,
    QualityScore,
    Summary,
    Caption,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(VisualFeatures::Table)
                .if_not_exists()
                .col(pk_auto(VisualFeatures::Id))
                .col(
                    ColumnDef::new(VisualFeatures::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(VisualFeatures::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(integer(VisualFeatures::FramePercentage)) // Implicit .not_null()
                .col(
                    ColumnDef::new(VisualFeatures::Embedding)
                        .vector(Some(768))
                        .not_null(),
                )
                .col(string(VisualFeatures::SceneType)) // Implicit .not_null()
                .col(string_null(VisualFeatures::PeopleType))
                .col(string_null(VisualFeatures::AnimalType))
                .col(string_null(VisualFeatures::DocumentType))
                .col(string_null(VisualFeatures::ObjectType))
                .col(string_null(VisualFeatures::ActivityType))
                .col(string_null(VisualFeatures::EventType))
                .col(string_null(VisualFeatures::WeatherCondition))
                // Boolean columns
                .col(boolean(VisualFeatures::IsOutside)) // Implicit .not_null()
                .col(boolean(VisualFeatures::IsLandscape))
                .col(boolean(VisualFeatures::IsCityscape))
                .col(boolean(VisualFeatures::IsTravel))
                .col(boolean(VisualFeatures::HasLegibleText))
                // Text columns
                .col(text_null(VisualFeatures::OcrText))
                .col(text_null(VisualFeatures::DocumentSummary))
                // Quality metrics
                .col(float(VisualFeatures::MeasuredSharpness))
                .col(integer(VisualFeatures::MeasuredNoise))
                .col(float(VisualFeatures::MeasuredBrightness))
                .col(float(VisualFeatures::MeasuredContrast))
                .col(float(VisualFeatures::MeasuredClipping))
                .col(float(VisualFeatures::MeasuredDynamicRange))
                .col(float(VisualFeatures::QualityScore))
                // Descriptive fields
                .col(string_null(VisualFeatures::Summary))
                .col(string(VisualFeatures::Caption))
                .to_owned(),
        )
        .await?;

        // Create vector index
        let db = m.get_connection();
        db.execute_unprepared(
            r"
                CREATE INDEX emb_idx ON visual_features
                USING hnsw (embedding vector_cosine_ops)
                WITH (m = 16, ef_construction = 200)
                ",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(VisualFeatures::Table).to_owned())
            .await
    }
}
