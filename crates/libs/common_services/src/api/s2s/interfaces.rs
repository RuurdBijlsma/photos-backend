use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct S2SInviteSummaryRequest {
    /// The full invitation token
    pub token: String,
}

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DownloadParams {
    pub relative_path: String,
}
