use crate::api::album::error::AlbumError;
use crate::api::album::interfaces::AlbumMediaItemSummary;
use crate::database::DbError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::postgres::PgQueryResult;
use sqlx::{Executor, FromRow, Postgres};
use utoipa::ToSchema;


/// Represents the link between a media item and an album.
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMediaItem {
    pub album_id: String,
    pub media_item_id: String,
    pub added_by_user: Option<i32>,
    pub added_at: DateTime<Utc>,
}
