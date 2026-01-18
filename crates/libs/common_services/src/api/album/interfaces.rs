use crate::api::timeline::interfaces::SortDirection;
use crate::database::album::album::AlbumRole;
use chrono::{DateTime, Utc};
use common_types::pb::api::TimelineItem;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
// --- Request Payloads ---

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlbumRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub media_item_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddMediaToAlbumRequest {
    pub media_item_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AddCollaboratorRequest {
    pub user_email: String,
    pub role: AlbumRole,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAlbumRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub thumbnail_id: Option<String>,
    pub is_public: Option<bool>,
}

// --- Request Payloads for Cross-Server Sharing ---

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckInviteRequest {
    /// The full invitation token string (e.g., "inv-...")
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumIdParams {
    pub album_id: String,
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMediaParams {
    pub album_id: String,
    pub media_item_id: String,
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveCollaboratorParams {
    pub album_id: String,
    pub collaborator_id: i64,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListAlbumsParam {
    #[serde(default)]
    pub sort_direction: SortDirection,
    #[serde(default)]
    pub sort_field: AlbumSortField,
}

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAlbumMediaParams {
    /// Comma separated list of Rank IDs (Start Ranks of the groups).
    pub groups: String,
}

// --- Response Payloads ---

/// Full details of an album, including its media items and collaborators.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetailsResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_id: Option<String>,
    pub is_public: bool,
    pub owner_id: i32,
    pub created_at: DateTime<Utc>,
    pub media_items: Vec<AlbumMediaItemSummary>,
    pub collaborators: Vec<CollaboratorSummary>,
}

/// A summary of a media item within an album.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMediaItemSummary {
    pub media_item: TimelineItem,
    pub added_at: DateTime<Utc>,
}

/// A summary of a collaborator on an album.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollaboratorSummary {
    pub id: i64,
    pub name: String,
    pub role: AlbumRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumShareClaims {
    pub iss: String, // Issuer (server's public_url)
    pub sub: String, // Subject (album_id)
    pub exp: i64,    // Expiration time (as a Unix timestamp)
    pub sharer_username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Default, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AlbumSortField {
    #[default]
    UpdatedAt,
    LatestPhoto,
    Name,
}

impl AlbumSortField {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::UpdatedAt => "updated_at",
            Self::LatestPhoto => "latest_photo",
        }
    }
}
