use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RandomPhotoResponse {
    pub media_id: String,
    pub themes: Option<Vec<Value>>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetMediaByMonthParams {
    /// "YYYY-MM-DD" strings.
    pub months: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetMediaItemParams {
    pub id: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColorThemeParams {
    pub color: String,
}
