use crate::sea_orm::EnumIter;
use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[derive(Iden, EnumIter)]
pub enum ObjectBoxes {
    Table,
    Id,
    UpdatedAt,
    CreatedAt,
    Position,
    Width,
    Height,
    Confidence,
    Label,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(ObjectBoxes::Table)
                .if_not_exists()
                .col(pk_auto(ObjectBoxes::Id))
                .col(
                    ColumnDef::new(ObjectBoxes::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(ObjectBoxes::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(ObjectBoxes::Position)
                        .array(ColumnType::Float)
                        .not_null(),
                )
                .col(ColumnDef::new(ObjectBoxes::Width).float().not_null())
                .col(ColumnDef::new(ObjectBoxes::Height).float().not_null())
                .col(ColumnDef::new(ObjectBoxes::Label).string().not_null())
                .col(ColumnDef::new(ObjectBoxes::Confidence).float().not_null())
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(ObjectBoxes::Table).to_owned())
            .await
    }
}
