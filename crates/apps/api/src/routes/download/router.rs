use crate::api_state::ApiContext;
use crate::download::handlers::download_full_file;
use axum::{Router, routing::get};

pub fn download_protected_router() -> Router<ApiContext> {
    Router::new().route("/download/full-file", get(download_full_file))
}
