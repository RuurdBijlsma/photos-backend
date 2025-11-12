use axum::extract::FromRef;
use sqlx::PgPool;

// The #[derive(Clone)] is crucial for Axum to share the state with all handlers.
#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub http_client: reqwest::Client,
}

// These impls allow Axum to extract the PgPool and reqwest::Client from the AppState.
// This is useful for middleware and extractors that might only need one part of the state.
impl FromRef<ApiState> for PgPool {
    fn from_ref(state: &ApiState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<ApiState> for reqwest::Client {
    fn from_ref(state: &ApiState) -> Self {
        state.http_client.clone()
    }
}
