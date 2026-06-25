use crate::api_state::ApiContext;
use crate::jobs::handlers::job_summary_handler;
use axum::{routing::get, Router};

pub fn jobs_admin_router() -> Router<ApiContext> {
    Router::new()
        .route("/jobs", get(job_summary_handler))
}
