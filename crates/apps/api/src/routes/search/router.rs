use crate::api_state::ApiContext;
use crate::search::handlers::{
    get_random_search_suggestion_handler, get_search_results, get_search_suggestions_handler,
};
use axum::{Router, routing::get};

pub fn search_protected_router() -> Router<ApiContext> {
    Router::new()
        .route("/search", get(get_search_results))
        .route("/search/suggestions", get(get_search_suggestions_handler))
        .route(
            "/search/suggestions/random",
            get(get_random_search_suggestion_handler),
        )
}
