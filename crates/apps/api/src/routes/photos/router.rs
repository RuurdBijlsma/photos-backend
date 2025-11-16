use crate::api_state::ApiState;
use crate::photos::handlers::{
    get_color_theme_handler, get_full_item_handler, get_photos_by_month_handler, get_random_photo,
    get_timeline_ids_handler, get_timeline_ratios_handler,
};
use axum::{Router, routing::get};

pub fn photos_protected_router() -> Router<ApiState> {
    Router::new()
        .route("/photos/random", get(get_random_photo))
        .route("/photos/theme", get(get_color_theme_handler))
        .route("/photos/timeline/ratios", get(get_timeline_ratios_handler))
        .route("/photos/timeline/ids", get(get_timeline_ids_handler))
        .route("/photos/by-month", get(get_photos_by_month_handler))
        .route("/photos/item", get(get_full_item_handler))
}
