// crates/api/src/routes/photos/interfaces.rs

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
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
    /// "YYYY-MM" strings.
    pub months: String,
}
