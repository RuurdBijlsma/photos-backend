use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // VisualFeatures remains unchanged.
        manager
            .alter_table(
                Table::alter()
                    .table(VisualFeatures::Table)
                    .add_column(
                        ColumnDef::new(VisualFeatures::ImageId)
                            .string_len(22)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_visual_features_image_id")
                    .from(VisualFeatures::Table, VisualFeatures::ImageId)
                    .to(Images::Table, Images::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // --- one-to-one relationships ---

        // GPS: Add column to GPS: and create FK.
        manager
            .alter_table(
                Table::alter()
                    .table(Gps::Table)
                    .add_column(
                        ColumnDef::new(Gps::ImageId)
                            .string_len(22)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_image_gps_id")
                    .from(Gps::Table, Gps::ImageId)
                    .to(Images::Table, Images::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Metadata: Add column to Metadata and create FK.
        manager
            .alter_table(
                Table::alter()
                    .table(Metadata::Table)
                    .add_column(
                        ColumnDef::new(Metadata::ImageId)
                            .string_len(22)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_image_metadata_id")
                    .from(Metadata::Table, Metadata::ImageId)
                    .to(Images::Table, Images::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Tags: Add column to Tags and create FK.
        manager
            .alter_table(
                Table::alter()
                    .table(Tags::Table)
                    .add_column(
                        ColumnDef::new(Tags::ImageId)
                            .string_len(22)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_image_tags_id")
                    .from(Tags::Table, Tags::ImageId)
                    .to(Images::Table, Images::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Weather: Add column to Weather and create FK.
        manager
            .alter_table(
                Table::alter()
                    .table(Weather::Table)
                    .add_column(
                        ColumnDef::new(Weather::ImageId)
                            .string_len(22)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_image_weather_id")
                    .from(Weather::Table, Weather::ImageId)
                    .to(Images::Table, Images::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Weather
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_image_weather_id")
                    .table(Weather::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Weather::Table)
                    .drop_column(Weather::ImageId)
                    .to_owned(),
            )
            .await?;

        // Tags
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_image_tags_id")
                    .table(Tags::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tags::Table)
                    .drop_column(Tags::ImageId)
                    .to_owned(),
            )
            .await?;

        // Metadata
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_image_metadata_id")
                    .table(Metadata::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Metadata::Table)
                    .drop_column(Metadata::ImageId)
                    .to_owned(),
            )
            .await?;

        // GPS
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_image_gps_id")
                    .table(Gps::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Gps::Table)
                    .drop_column(Gps::ImageId)
                    .to_owned(),
            )
            .await?;

        // VisualFeatures
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_visual_features_image_id")
                    .table(VisualFeatures::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(VisualFeatures::Table)
                    .drop_column(VisualFeatures::ImageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Images {
    Table,
    Id,
}

#[derive(Iden)]
enum VisualFeatures {
    Table,
    ImageId,
}

#[derive(Iden)]
enum Gps {
    Table,
    ImageId,
}

#[derive(Iden)]
enum Metadata {
    Table,
    ImageId,
}

#[derive(Iden)]
enum Tags {
    Table,
    ImageId,
}

#[derive(Iden)]
enum Weather {
    Table,
    ImageId,
}
