use crate::api_state::ApiState;
use crate::root::handlers::root;
use axum::{Router, routing::get};

pub fn root_public_router() -> Router<ApiState> {
    Router::new().route("/", get(root))
}
