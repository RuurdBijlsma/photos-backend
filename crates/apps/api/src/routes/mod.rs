// crates/api/rc/routes/mod.rs

pub mod album;
mod api_doc;
pub mod auth;
pub mod download;
pub mod onboarding;
pub mod photos;
pub mod root;
pub mod s2s;

use crate::album::router::{album_auth_optional_router, album_protected_router};
use crate::api_state::ApiState;
use crate::auth::middleware::{ApiUser, OptionalUser, require_role};
use crate::auth::router::{auth_protected_router, auth_public_router};
use crate::download::router::download_protected_router;
use crate::onboarding::router::onboarding_admin_routes;
use crate::photos::router::photos_protected_router;
use crate::root::handlers::root;
use crate::root::router::root_public_router;
use crate::routes::api_doc::ApiDoc;
use crate::s2s::router::s2s_public_router;
use axum::Router;
use axum::middleware::{from_extractor_with_state, from_fn_with_state};
use common_services::database::app_user::UserRole;
use tower_http::{LatencyUnit, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// --- Router Construction ---
pub fn create_router(api_state: ApiState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(public_routes())
        .merge(protected_routes(api_state.clone()))
        .merge(auth_optional_routes(api_state.clone()))
        .merge(admin_routes(api_state.clone()))
        .with_state(api_state)
        .layer(
            TraceLayer::new_for_http().on_response(
                tower_http::trace::DefaultOnResponse::new()
                    .level(tracing::Level::INFO)
                    .latency_unit(LatencyUnit::Millis),
            ),
        )
}

fn public_routes() -> Router<ApiState> {
    Router::new()
        .merge(auth_public_router())
        .merge(root_public_router())
        .merge(s2s_public_router())
}

fn auth_optional_routes(api_state: ApiState) -> Router<ApiState> {
    Router::new()
        .merge(album_auth_optional_router())
        .route_layer(from_extractor_with_state::<OptionalUser, ApiState>(
            api_state,
        ))
}

fn protected_routes(api_state: ApiState) -> Router<ApiState> {
    Router::new()
        .merge(auth_protected_router())
        .merge(download_protected_router())
        .merge(photos_protected_router())
        .merge(album_protected_router())
        .route_layer(from_extractor_with_state::<ApiUser, ApiState>(api_state))
}

fn admin_routes(api_state: ApiState) -> Router<ApiState> {
    Router::new()
        .merge(onboarding_admin_routes())
        .route_layer(from_fn_with_state(UserRole::Admin, require_role))
        .route_layer(from_extractor_with_state::<ApiUser, ApiState>(api_state))
}
