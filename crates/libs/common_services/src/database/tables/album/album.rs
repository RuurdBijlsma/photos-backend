use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Represents a single album in the database, with count of media items.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumWithCount {
    pub id: String,
    pub owner_id: i32,
    pub name: String,
    pub thumbnail_id: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub media_count: i32,
}

/// Represents a single album in the database.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub owner_id: i32,
    pub name: String,
    pub thumbnail_id: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AlbumWithCount> for Album {
    fn from(album: AlbumWithCount) -> Self {
        Self {
            id: album.id,
            owner_id: album.owner_id,
            name: album.name,
            thumbnail_id: album.thumbnail_id,
            description: album.description,
            is_public: album.is_public,
            created_at: album.created_at,
            updated_at: album.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlbumSummary {
    pub name: String,
    pub description: Option<String>,
    pub relative_paths: Vec<String>,
}
