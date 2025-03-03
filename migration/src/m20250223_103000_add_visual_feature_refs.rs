use crate::sea_orm::EnumIter;
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
                    .add_column(
                        ColumnDef::new(FaceBoxes::VisualFeatureId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        let face_fk = ForeignKey::create()
            .name("fk_face_boxes_visual_feature_id")
            .from(FaceBoxes::Table, FaceBoxes::VisualFeatureId)
            .to(VisualFeatures::Table, VisualFeatures::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .to_owned();
        manager.create_foreign_key(face_fk).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OcrBoxes::Table)
                    .add_column(
                        ColumnDef::new(OcrBoxes::VisualFeatureId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        let ocr_fk = ForeignKey::create()
            .name("fk_ocr_boxes_visual_feature_id")
            .from(OcrBoxes::Table, OcrBoxes::VisualFeatureId)
            .to(VisualFeatures::Table, VisualFeatures::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .to_owned();
        manager.create_foreign_key(ocr_fk).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjectBoxes::Table)
                    .add_column(
                        ColumnDef::new(ObjectBoxes::VisualFeatureId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        let object_fk = ForeignKey::create()
            .name("fk_object_boxes_visual_feature_id")
            .from(ObjectBoxes::Table, ObjectBoxes::VisualFeatureId)
            .to(VisualFeatures::Table, VisualFeatures::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .to_owned();
        manager.create_foreign_key(object_fk).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let object_fk = ForeignKey::drop()
            .name("fk_object_boxes_visual_feature_id")
            .table(ObjectBoxes::Table)
            .to_owned();
        manager.drop_foreign_key(object_fk).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ObjectBoxes::Table)
                    .drop_column(ObjectBoxes::VisualFeatureId)
                    .to_owned(),
            )
            .await?;

        let ocr_fk = ForeignKey::drop()
            .name("fk_ocr_boxes_visual_feature_id")
            .table(OcrBoxes::Table)
            .to_owned();
        manager.drop_foreign_key(ocr_fk).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(OcrBoxes::Table)
                    .drop_column(OcrBoxes::VisualFeatureId)
                    .to_owned(),
            )
            .await?;

        let face_fk = ForeignKey::drop()
            .name("fk_face_boxes_visual_feature_id")
            .table(FaceBoxes::Table)
            .to_owned();
        manager.drop_foreign_key(face_fk).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(FaceBoxes::Table)
                    .drop_column(FaceBoxes::VisualFeatureId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum FaceBoxes {
    Table,
    VisualFeatureId,
}

#[derive(Iden)]
enum OcrBoxes {
    Table,
    VisualFeatureId,
}

#[derive(Iden)]
enum ObjectBoxes {
    Table,
    VisualFeatureId,
}

#[derive(Iden, EnumIter)]
pub enum VisualFeatures {
    Table,
    Id,
}
