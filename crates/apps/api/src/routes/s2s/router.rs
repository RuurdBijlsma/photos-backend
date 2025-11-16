use crate::api_state::ApiState;
use crate::s2s::handlers::{download_file_handler, invite_summary_handler};
use axum::{Router, routing::get};

pub fn s2s_public_router() -> Router<ApiState> {
    Router::new()
        .route("/s2s/albums/invite-summary", get(invite_summary_handler))
        .route("/s2s/albums/files", get(download_file_handler))
}
