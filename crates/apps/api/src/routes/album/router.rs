use crate::api_state::ApiContext;
use crate::routes::album::handlers::{
    accept_invite_handler, add_collaborator_handler, add_media_to_album_handler,
    check_invite_handler, create_album_handler, generate_invite_handler, get_album_details_handler,
    get_user_albums_handler, remove_collaborator_handler, remove_media_from_album_handler,
    update_album_handler,
};
use axum::routing::put;
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn album_auth_optional_router() -> Router<ApiContext> {
    Router::new().route("/album/{album_id}", get(get_album_details_handler))
}

pub fn album_protected_router() -> Router<ApiContext> {
    Router::new()
        .route(
            "/album",
            post(create_album_handler).get(get_user_albums_handler),
        )
        .route("/album/{album_id}", put(update_album_handler))
        .route("/album/{album_id}/media", post(add_media_to_album_handler))
        .route(
            "/album/{album_id}/media/{media_item_id}",
            delete(remove_media_from_album_handler),
        )
        .route(
            "/album/{album_id}/collaborators",
            post(add_collaborator_handler),
        )
        .route(
            "/album/{album_id}/collaborators/{collaborator_id}",
            delete(remove_collaborator_handler),
        )
        .route(
            "/album/{album_id}/generate-invite",
            get(generate_invite_handler),
        )
        .route("/album/invite/check", post(check_invite_handler))
        .route("/album/invite/accept", post(accept_invite_handler))
}
