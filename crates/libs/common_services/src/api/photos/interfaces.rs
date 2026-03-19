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
pub struct GetMediaItemParams {
    pub id: String,
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
