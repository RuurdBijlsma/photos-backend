use crate::database::UpdateField;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RandomPhotoResponse {
    pub media_id: String,
    pub themes: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ColorThemeParams {
    pub color: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadMediaParams {
    pub path: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PhotoThumbnailParams {
    pub size: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMediaItemRequest {
    #[serde(default)]
    pub user_caption: UpdateField<String>,
    #[serde(default)]
    pub taken_at_local: Option<String>,
    #[serde(default)]
    pub use_panorama_viewer: Option<bool>,
    #[serde(default)]
    pub timezone_offset_seconds: UpdateField<i32>,
}
