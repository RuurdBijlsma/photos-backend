use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePersonRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PersonSummary {
    pub id: i64,
    pub name: Option<String>,
    pub thumbnail_media_item_id: Option<String>,
    pub photo_count: i32,
}
