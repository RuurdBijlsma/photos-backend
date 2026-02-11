use crate::api_state::ApiContext;

use crate::photos::handlers::{
    download_full_file, get_color_theme_handler, get_full_item_handler, get_photo_thumbnail,
    get_random_photo,
};
use axum::{Router, routing::get};

pub fn photos_protected_router() -> Router<ApiContext> {
    Router::new()
        .route("/photos/random", get(get_random_photo))
        .route("/photos/theme", get(get_color_theme_handler))
        .route("/photos/item", get(get_full_item_handler))
        .route("/photos/download", get(download_full_file))
}

pub fn photos_public_router() -> Router<ApiContext> {
    Router::new().route(
        "/photos/thumbnail/{media_item_id}",
        get(get_photo_thumbnail),
    )
}
