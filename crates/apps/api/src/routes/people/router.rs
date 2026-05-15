use crate::api_state::ApiContext;
use crate::routes::people::handlers::{
    get_person_photos_handler, list_people_handler, update_person_handler,
};
use axum::Router;
use axum::routing::get;

pub fn people_protected_router() -> Router<ApiContext> {
    Router::new()
        .route("/people", get(list_people_handler))
        .route(
            "/people/{id}",
            get(get_person_photos_handler).patch(update_person_handler),
        )
}
