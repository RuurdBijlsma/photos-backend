use crate::api_state::ApiContext;
use crate::trash::handlers::{
    get_trash_handler, perma_delete_handler, restore_handler, soft_delete_handler,
};
use axum::{Router, routing::get, routing::post, routing::delete};

pub fn trash_protected_router() -> Router<ApiContext> {
    Router::new()
        .route("/trash", get(get_trash_handler))
        .route("/trash/soft-delete", delete(soft_delete_handler))
        .route("/trash/hard-delete", delete(perma_delete_handler))
        .route("/trash/restore", post(restore_handler))
}
