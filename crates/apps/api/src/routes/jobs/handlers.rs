use crate::api_state::ApiContext;
use axum::Json;
use axum::extract::{State};
use axum_extra::extract::Query;
use common_services::api::app_error::AppError;
use common_services::api::jobs::interfaces::{JobsQuery, PaginatedJobsResponse};
use common_services::api::jobs::service::get_job_overview;
use tracing::instrument;

#[instrument(skip(context), err(Debug))]
pub async fn job_summary_handler(
    State(context): State<ApiContext>,
    Query(query): Query<JobsQuery>,
) -> Result<Json<PaginatedJobsResponse>, AppError> {
    let overview = get_job_overview(&context.pool, query).await?;
    Ok(Json(overview))
}