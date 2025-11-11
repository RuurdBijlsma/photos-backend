use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportAlbumPayload {
    pub token: String,
    pub album_name: String,
    pub album_description: Option<String>,
    pub remote_owner_identity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportAlbumItemPayload {
    pub remote_media_item_id: String,
    pub token: String,
    pub local_album_id: String,
    pub remote_owner_identity: String,
    pub remote_server_url: String,
}