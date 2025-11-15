use axum::extract::FromRef;
use common_services::s2s_client::S2SClient;
use sqlx::PgPool;

// The #[derive(Clone)] is crucial for Axum to share the state with all handlers.
#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub s2s_client: S2SClient,
}

// These impls allow Axum to extract the PgPool and reqwest::Client from the AppState.
// This is useful for middleware and extractors that might only need one part of the state.
impl FromRef<ApiState> for PgPool {
    fn from_ref(state: &ApiState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<ApiState> for S2SClient {
    fn from_ref(state: &ApiState) -> Self {
        state.s2s_client.clone()
    }
}
