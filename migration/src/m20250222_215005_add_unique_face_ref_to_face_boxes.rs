use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(FaceBoxes::Table)
                    .add_column(ColumnDef::new(FaceBoxes::UniqueFaceId).integer())
                    .to_owned(),
            )
            .await?;

        let foreign_key = ForeignKey::create()
            .name("fk_face_boxes_unique_face_id")
            .from(FaceBoxes::Table, FaceBoxes::UniqueFaceId)
            .to(UniqueFaces::Table, UniqueFaces::Id)
            .on_delete(ForeignKeyAction::NoAction)
            .on_update(ForeignKeyAction::NoAction)
            .to_owned();

        manager.create_foreign_key(foreign_key).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let foreign_key = ForeignKey::drop()
            .name("fk_face_boxes_unique_face_id")
            .table(FaceBoxes::Table)
            .to_owned();

        manager.drop_foreign_key(foreign_key).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(FaceBoxes::Table)
                    .drop_column(FaceBoxes::UniqueFaceId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum FaceBoxes {
    Table,
    UniqueFaceId,
}

#[derive(Iden)]
enum UniqueFaces {
    Table,
    Id,
}
