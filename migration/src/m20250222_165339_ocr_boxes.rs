use crate::sea_orm::EnumIter;
use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[derive(Iden, EnumIter)]
pub enum OcrBoxes {
    Table,
    Id,
    UpdatedAt,
    CreatedAt,
    Position,
    Width,
    Height,
    Confidence,
    Text,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(OcrBoxes::Table)
                .if_not_exists()
                .col(pk_auto(OcrBoxes::Id))
                .col(
                    ColumnDef::new(OcrBoxes::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(OcrBoxes::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(OcrBoxes::Position)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(ColumnDef::new(OcrBoxes::Width).float().not_null())
                .col(ColumnDef::new(OcrBoxes::Height).float().not_null())
                .col(ColumnDef::new(OcrBoxes::Text).string().not_null())
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(OcrBoxes::Table).to_owned())
            .await
    }
}
