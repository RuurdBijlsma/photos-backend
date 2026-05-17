use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePersonRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PersonSummary {
    pub id: String,
    pub name: Option<String>,
    pub face_thumb_id: Option<String>,
    pub face_cluster_ids: Vec<String>,
    pub photo_count: i32,
}
