use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use crate::database::album::AlbumRole;

/// Represents a user's role in an album (a collaborator).
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCollaborator {
    pub id: i64,
    pub album_id: String,
    pub user_id: Option<i32>,
    pub remote_user_id: Option<String>,
    pub role: AlbumRole,
    pub added_at: DateTime<Utc>,
}