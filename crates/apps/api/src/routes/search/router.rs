use crate::api_state::ApiContext;
use crate::search::handlers::get_search_results;
use axum::{routing::get, Router};

pub fn search_protected_router() -> Router<ApiContext> {
    Router::new()
        .route("/search", get(get_search_results))
}