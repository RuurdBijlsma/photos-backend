use crate::api_state::ApiContext;
use crate::auth::handlers::{get_me, login, logout, refresh_session, register};
use axum::{
    routing::{get, post},
    Router,
};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tracing::info;
use app_state::RateLimitingSettings;

pub fn auth_public_router(rate_limiting: &RateLimitingSettings) -> Router<ApiContext> {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(rate_limiting.req_per_second)
        .burst_size(rate_limiting.burst_size)
        .finish()
        .expect("Could not create rate-limiting governor.");

    info!("Using request limits: rate_limiting.req_per_second {:?}", rate_limiting.req_per_second);
    info!("Using request limits: rate_limiting.burst_size{:?}", rate_limiting.burst_size);
    info!("Using request limits: rate_limiting.req_per_second {:?}", rate_limiting.req_per_second);
    info!("Using request limits: rate_limiting.burst_size{:?}", rate_limiting.burst_size);
    info!("Using request limits: rate_limiting.req_per_second {:?}", rate_limiting.req_per_second);
    info!("Using request limits: rate_limiting.burst_size{:?}", rate_limiting.burst_size);

    Router::new()
        .route("/auth/refresh", post(refresh_session))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .layer(GovernorLayer::new(governor_conf))
}

pub fn auth_protected_router() -> Router<ApiContext> {
    Router::new().route("/auth/me", get(get_me))
}
