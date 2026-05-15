use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub has_clustered_people: bool,
    pub has_clustered_photos: bool,
}
