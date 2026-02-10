use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub result: String,
}
