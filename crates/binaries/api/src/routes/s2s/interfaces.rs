use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct S2SInviteSummaryRequest {
    /// The full invitation token string (e.g., "inv-...")
    pub token: String,
}