use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "album_role", rename_all = "lowercase")]
pub enum AlbumRole {
    Owner,
    Contributor,
    Viewer,
}

impl Display for AlbumRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Owner => "owner",
            Self::Contributor => "contributor",
            Self::Viewer => "viewer",
        };
        f.write_str(s)
    }
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
