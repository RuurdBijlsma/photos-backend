use chrono::{DateTime, Utc};
use common_photos::UserRole;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents a user in the application.
#[derive(Debug, Serialize, FromRow, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub name: String,
    pub media_folder: Option<String>,
    pub role: UserRole,
}
