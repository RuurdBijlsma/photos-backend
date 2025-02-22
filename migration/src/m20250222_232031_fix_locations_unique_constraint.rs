use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
#[derive(Iden)]
enum Locations {
    Table,
    Country,
    Province,
    City,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_index(
            Index::create()
                .name("unique_location")
                .table(Locations::Table)
                .col(Locations::City)
                .col(Locations::Province)
                .col(Locations::Country)
                .unique()
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(Index::drop().name("unique_location").to_owned())
            .await
    }
}
