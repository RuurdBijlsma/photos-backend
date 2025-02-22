use crate::sea_orm::EnumIter;
use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[derive(Iden, EnumIter)]
pub enum FaceBoxes {
    Table,
    Id,
    UpdatedAt,
    CreatedAt,
    Position,
    Width,
    Height,
    Age,
    Confidence,
    Sex,
    MouthLeft,
    MouthRight,
    NoseTip,
    EyeLeft,
    EyeRight,
    Embedding,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(FaceBoxes::Table)
                .if_not_exists()
                .col(pk_auto(FaceBoxes::Id))
                .col(
                    ColumnDef::new(FaceBoxes::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(FaceBoxes::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(FaceBoxes::Position)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(ColumnDef::new(FaceBoxes::Width).float().not_null())
                .col(ColumnDef::new(FaceBoxes::Height).float().not_null())
                .col(
                    ColumnDef::new(FaceBoxes::MouthLeft)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(FaceBoxes::MouthRight)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(FaceBoxes::NoseTip)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(FaceBoxes::EyeLeft)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(FaceBoxes::EyeRight)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(FaceBoxes::Embedding)
                        .vector(Some(512))
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;

        // Create vector index
        let db = m.get_connection();
        db.execute_unprepared(
            r#"
                CREATE INDEX face_box_emb_idx ON face_boxes
                USING hnsw (embedding vector_cosine_ops)
                WITH (m = 16, ef_construction = 200)
                "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(FaceBoxes::Table).to_owned())
            .await
    }
}
