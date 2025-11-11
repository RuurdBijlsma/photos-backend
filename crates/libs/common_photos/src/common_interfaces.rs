use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A summary of an album invitation, sent from the sharing server
/// to the receiving server and then to the frontend.
#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InviteSummaryResponse {
    pub album_name: String,
    pub album_description: Option<String>,
    /// The list of media item IDs that are part of this album.
    pub media_item_ids: Vec<String>,
}