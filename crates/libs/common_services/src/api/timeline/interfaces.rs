use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetMediaByMonthParams {
    /// "YYYY-MM-DD" strings.
    pub months: String,
}
