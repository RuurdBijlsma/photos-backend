use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // one User has many Images.
        m.alter_table(
            Table::alter()
                .table(Images::Table)
                .add_column(ColumnDef::new(Images::UserId).integer().not_null())
                .to_owned(),
        )
        .await?;

        m.create_foreign_key(
            ForeignKey::create()
                .name("fk_images_user_id")
                .from(Images::Table, Images::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_foreign_key(
            ForeignKey::drop()
                .name("fk_images_user_id")
                .table(Images::Table)
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Images::Table)
                .drop_column(Images::UserId)
                .to_owned(),
        )
        .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Images {
    Table,
    UserId,
}
