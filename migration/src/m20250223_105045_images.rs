use sea_orm::EnumIter;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden, EnumIter)]
enum Images {
    Table,
    Id,
    UpdatedAt,
    CreatedAt,
    Filename,
    RelativePath,
    Hash,
    Width,
    Height,
    Duration,
    Format,
    SizeBytes,
    DatetimeLocal,
    DatetimeUtc,
    DatetimeSource,
    TimezoneName,
    TimezoneOffset,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r"
CREATE OR REPLACE FUNCTION short_uuid() RETURNS text AS $$
DECLARE
    uuid_bytes bytea;
    encoded text;
BEGIN
    uuid_bytes = uuid_send(gen_random_uuid());
    encoded = encode(uuid_bytes, 'base64');
    encoded = replace(encoded, '+', '-');
    encoded = replace(encoded, '/', '_');
    encoded = rtrim(encoded, '=');
    RETURN encoded;
END;
$$ LANGUAGE plpgsql;
        ",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Images::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Images::Id)
                            .string_len(22)
                            .primary_key()
                            .default(Expr::cust("short_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Images::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Images::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(ColumnDef::new(Images::Filename).string().not_null())
                    .col(
                        ColumnDef::new(Images::RelativePath)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Images::Hash).string().not_null())
                    .col(ColumnDef::new(Images::Width).integer().not_null())
                    .col(ColumnDef::new(Images::Height).integer().not_null())
                    .col(ColumnDef::new(Images::Duration).float().null())
                    .col(ColumnDef::new(Images::Format).string().not_null())
                    .col(ColumnDef::new(Images::SizeBytes).integer().not_null())
                    .col(ColumnDef::new(Images::DatetimeLocal).timestamp().not_null())
                    .col(ColumnDef::new(Images::DatetimeUtc).timestamp().null())
                    .col(ColumnDef::new(Images::DatetimeSource).string().not_null())
                    .col(ColumnDef::new(Images::TimezoneName).string().null())
                    .col(ColumnDef::new(Images::TimezoneOffset).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_images_datetime_local")
                    .table(Images::Table)
                    .col(Images::DatetimeLocal)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_images_filename")
                    .table(Images::Table)
                    .col(Images::Filename)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Images::Table).to_owned())
            .await
    }
}
