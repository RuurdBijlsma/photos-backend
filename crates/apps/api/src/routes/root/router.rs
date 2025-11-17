use crate::api_state::ApiContext;
use crate::root::handlers::root;
use axum::{Router, routing::get};

pub fn root_public_router() -> Router<ApiContext> {
    Router::new().route("/", get(root))
}
