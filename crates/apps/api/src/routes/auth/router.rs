use crate::api_state::ApiContext;
use crate::auth::handlers::{get_me, login, logout, refresh_session, register};
use axum::{
    Router,
    routing::{get, post},
};

pub fn auth_public_router() -> Router<ApiContext> {
    Router::new()
        .route("/auth/refresh", post(refresh_session))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
}

pub fn auth_protected_router() -> Router<ApiContext> {
    Router::new().route("/auth/me", get(get_me))
}
