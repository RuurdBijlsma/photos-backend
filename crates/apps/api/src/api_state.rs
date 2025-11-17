use app_state::{AppSettings, IngestSettings};
use axum::extract::FromRef;
use common_services::s2s_client::S2SClient;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ApiContext {
    pub pool: PgPool,
    pub s2s_client: S2SClient,
    pub settings: AppSettings,
}

// These impls allow Axum to extract the PgPool and reqwest::Client from the AppState.
// This is useful for middleware and extractors that might only need one part of the state.
impl FromRef<ApiContext> for PgPool {
    fn from_ref(state: &ApiContext) -> Self {
        state.pool.clone()
    }
}

impl FromRef<ApiContext> for S2SClient {
    fn from_ref(state: &ApiContext) -> Self {
        state.s2s_client.clone()
    }
}

impl FromRef<ApiContext> for AppSettings {
    fn from_ref(state: &ApiContext) -> Self {
        state.settings.clone()
    }
}

impl FromRef<ApiContext> for IngestSettings {
    fn from_ref(state: &ApiContext) -> Self {
        state.settings.ingest.clone()
    }
}
