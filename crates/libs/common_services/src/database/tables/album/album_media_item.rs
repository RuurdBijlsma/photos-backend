use crate::api::album::error::AlbumError;
use crate::api::album::interfaces::AlbumMediaItemSummary;
use crate::database::DbError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::postgres::PgQueryResult;
use sqlx::{Executor, FromRow, Postgres};
use utoipa::ToSchema;

pub async fn remove_album_media_items(
    executor: impl Executor<'_, Database = Postgres>,
    album_id: &str,
    media_item_id: &str,
) -> Result<PgQueryResult, DbError> {
    Ok(sqlx::query!(
        "DELETE FROM album_media_item WHERE album_id = $1 AND media_item_id = $2",
        album_id,
        media_item_id
    )
    .execute(executor)
    .await?)
}

pub async fn insert_album_media_items(
    executor: impl Executor<'_, Database = Postgres>,
    album_id: &str,
    media_item_id: &str,
    user_id: i32,
) -> Result<PgQueryResult, DbError> {
    // todo: make this accept multiple media item ids
    Ok(sqlx::query!(
        r#"
            INSERT INTO album_media_item (album_id, media_item_id, added_by_user)
            VALUES ($1, $2, $3)
            ON CONFLICT (album_id, media_item_id) DO NOTHING
            "#,
        album_id,
        media_item_id,
        user_id
    )
    .execute(executor)
    .await?)
}

pub async fn get_album_media_items(
    executor: impl Executor<'_, Database = Postgres>,
    album_id: &str,
) -> Result<Vec<AlbumMediaItemSummary>, AlbumError> {
    let items = sqlx::query_as!(
        AlbumMediaItemSummary,
        r#"
            SELECT media_item_id as id, added_at
            FROM album_media_item
            WHERE album_id = $1
            ORDER BY added_at DESC
            "#,
        album_id
    )
    .fetch_all(executor)
    .await?;

    Ok(items)
}

/// Represents the link between a media item and an album.
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMediaItem {
    pub album_id: String,
    pub media_item_id: String,
    pub added_by_user: Option<i32>,
    pub added_at: DateTime<Utc>,
}
