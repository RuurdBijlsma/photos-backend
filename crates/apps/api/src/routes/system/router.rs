use crate::api_state::ApiContext;
use axum::{
    routing::get,
    Router,
};
use crate::system::handlers::get_system_stats_handler;

pub fn system_protected_router() -> Router<ApiContext> {
    Router::new()
        .route("/system/stats", get(get_system_stats_handler))
}
