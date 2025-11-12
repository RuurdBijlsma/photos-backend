use crate::routes::album::db_model::AlbumRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// --- Request Payloads ---

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlbumRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddMediaToAlbumRequest {
    pub media_item_ids: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddCollaboratorRequest {
    pub user_email: String,
    pub role: AlbumRole,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAlbumRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

// --- Request Payloads for Cross-Server Sharing ---

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckInviteRequest {
    /// The full invitation token string (e.g., "inv-...")
    pub token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInviteRequest {
    /// The full invitation token string.
    pub token: String,
    /// The name for the new album on the local server, pre-filled but editable by the user.
    pub name: String,
    /// The description for the new album on the local server.
    pub description: Option<String>,
}

// --- URL/Path Parameters ---

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumIdParams {
    pub album_id: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMediaParams {
    pub album_id: String,
    pub media_item_id: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveCollaboratorParams {
    pub album_id: String,
    pub collaborator_id: i64,
}

// --- Response Payloads ---

/// Full details of an album, including its media items and collaborators.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetailsResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub owner_id: i32,
    pub created_at: DateTime<Utc>,
    pub media_items: Vec<AlbumMediaItemSummary>,
    pub collaborators: Vec<CollaboratorSummary>,
}

/// A summary of a media item within an album.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMediaItemSummary {
    pub id: String,
    pub added_at: DateTime<Utc>,
}

/// A summary of a collaborator on an album.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollaboratorSummary {
    pub id: i64,
    pub name: String, // User's name for display
    pub role: AlbumRole,
}
