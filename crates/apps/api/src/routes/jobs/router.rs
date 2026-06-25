use crate::api_state::ApiContext;
use crate::jobs::handlers::{cancel_job_handler, job_summary_handler, retry_job_handler};
use axum::{routing::{get, post}, Router};

pub fn jobs_admin_router() -> Router<ApiContext> {
    Router::new()
        .route("/jobs", get(job_summary_handler))
        .route("/jobs/{id}/cancel", post(cancel_job_handler))
        .route("/jobs/{id}/retry", post(retry_job_handler))
}