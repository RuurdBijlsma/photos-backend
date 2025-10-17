use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct DownloadMediaQuery {
    pub path: String,
}
