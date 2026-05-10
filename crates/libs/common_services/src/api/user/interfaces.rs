use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub photo_count: i64,
    pub video_count: i64,
    pub album_count: i64,
    pub shared_album_count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub avatar_id: Option<String>,
    pub stats: UserStats,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserProfileRequest {
    pub name: Option<String>,
    pub avatar_id: Option<String>,
}
