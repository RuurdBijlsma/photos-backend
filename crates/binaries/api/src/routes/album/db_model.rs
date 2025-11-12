use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// Custom types to match the ENUMs in your SQL schema.
// It's good practice to define these in a shared library if they are used elsewhere,
// but defining them here is fine for this module.
#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "album_role", rename_all = "lowercase")]
#[derive(PartialEq, Eq)]
pub enum AlbumRole {
    Owner,
    Contributor,
    Viewer,
}

/// Represents a single album in the database.
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub owner_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

/// Represents a secure, single-use invitation token for sharing an album.
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInvite {
    pub id: i64,
    pub album_id: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
