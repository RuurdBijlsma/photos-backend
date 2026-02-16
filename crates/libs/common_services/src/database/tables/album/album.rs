use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

#[derive(FromRow)]
pub struct AlbumTimelineInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_id: Option<String>,
    pub is_public: bool,
    pub owner_id: i32,
    pub created_at: DateTime<Utc>,
    pub first_date: Option<NaiveDateTime>,
    pub last_date: Option<NaiveDateTime>,
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
    pub manual_sort: bool,
    pub media_count: i32,
    pub latest_media_item_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlbumSummary {
    pub name: String,
    pub description: Option<String>,
    pub relative_paths: Vec<String>,
}
