use crate::sea_orm::EnumIter;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden, EnumIter)]
pub enum UniqueFaces {
    Table,
    Id,
    UpdatedAt,
    CreatedAt,
    Label,
    Centroid,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("CREATE EXTENSION IF NOT EXISTS vector").await?;
        m.create_table(
            Table::create()
                .table(UniqueFaces::Table)
                .if_not_exists()
                .col(pk_auto(UniqueFaces::Id))
                .col(string(UniqueFaces::Label))
                .col(
                    ColumnDef::new(UniqueFaces::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(UniqueFaces::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")),
                )
                .col(
                    ColumnDef::new(UniqueFaces::Centroid)
                        .vector(Some(512))
                        .not_null(),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(UniqueFaces::Table).to_owned())
            .await
    }
}
