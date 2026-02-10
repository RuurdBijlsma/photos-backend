pub mod album;
mod api_doc;
pub mod auth;
pub mod onboarding;
pub mod photos;
pub mod root;
pub mod s2s;
pub mod search;
pub mod timeline;

use crate::album::router::{album_auth_optional_router, album_protected_router};
use crate::api_state::ApiContext;
use crate::auth::middlewares::optional_user::OptionalUser;
use crate::auth::middlewares::require_role::require_role;
use crate::auth::middlewares::user::ApiUser;
use crate::auth::middlewares::websocket::WsUser;
use crate::auth::router::{auth_protected_router, auth_public_router};
use crate::onboarding::router::onboarding_admin_routes;
use crate::photos::router::photos_protected_router;
use crate::root::handlers::root;
use crate::root::router::root_public_router;
use crate::routes::api_doc::ApiDoc;
use crate::s2s::router::s2s_public_router;
use crate::search::router::search_protected_router;
use crate::timeline::router::{timeline_protected_router, timeline_websocket_router};
use app_state::RateLimitingSettings;
use axum::Router;
use axum::middleware::{from_extractor_with_state, from_fn_with_state};
use common_services::database::app_user::UserRole;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// --- Router Construction ---
pub fn create_router(api_state: ApiContext) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(public_routes(&api_state.settings.api.rate_limiting))
        .merge(websocket_routes(api_state.clone()))
        .merge(protected_routes(api_state.clone()))
        .merge(auth_optional_routes(api_state.clone()))
        .merge(admin_routes(api_state.clone()))
        .with_state(api_state)
}

fn public_routes(rate_limiting: &RateLimitingSettings) -> Router<ApiContext> {
    Router::new()
        .merge(auth_public_router(rate_limiting))
        .merge(root_public_router())
        .merge(s2s_public_router())
}

// New WebSocket Route Group
fn websocket_routes(api_state: ApiContext) -> Router<ApiContext> {
    Router::new()
        .merge(timeline_websocket_router())
        .route_layer(from_extractor_with_state::<WsUser, ApiContext>(api_state))
}

fn auth_optional_routes(api_state: ApiContext) -> Router<ApiContext> {
    Router::new()
        .merge(album_auth_optional_router())
        .route_layer(from_extractor_with_state::<OptionalUser, ApiContext>(
            api_state,
        ))
}

fn protected_routes(api_state: ApiContext) -> Router<ApiContext> {
    Router::new()
        .merge(auth_protected_router())
        .merge(photos_protected_router())
        .merge(timeline_protected_router())
        .merge(search_protected_router())
        .merge(album_protected_router())
        .route_layer(from_extractor_with_state::<ApiUser, ApiContext>(api_state))
}

fn admin_routes(api_state: ApiContext) -> Router<ApiContext> {
    Router::new()
        .merge(onboarding_admin_routes())
        .route_layer(from_fn_with_state(UserRole::Admin, require_role))
        .route_layer(from_extractor_with_state::<ApiUser, ApiContext>(api_state))
}
