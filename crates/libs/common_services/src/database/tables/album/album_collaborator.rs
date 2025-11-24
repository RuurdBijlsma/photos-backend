use crate::database::album::album::AlbumRole;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents a user's role in an album (a collaborator).
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCollaborator {
    pub id: i64,
    pub album_id: String,
    pub user_id: i32,
    pub role: AlbumRole,
    pub added_at: DateTime<Utc>,
}
