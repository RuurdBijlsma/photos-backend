use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use utoipa::ToSchema;

/// Represents a user in the application.
#[derive(Debug, Serialize, FromRow, Clone, ToSchema)]
pub struct User {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub media_folder: Option<String>,
    pub role: UserRole,
}

/// Maps to the `user_role` Postgres enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admin => write!(f, "ADMIN"),
            Self::User => write!(f, "USER"),
        }
    }
}

/// Represents a user record from db, including the password hash.
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserWithPassword {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub media_folder: Option<String>,
    pub role: UserRole,
    pub password: String,
}

/// Represents a temporary record for a media item downloaded from another server,
/// awaiting ingestion.
#[derive(Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingAlbumMediaItem {
    pub relative_path: String,
    pub album_id: String,
    pub remote_user_identity: String,
}
