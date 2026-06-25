use crate::api_state::ApiContext;
use axum::Json;
use axum::extract::{Path, State};
use axum_extra::extract::Query;
use common_services::api::app_error::AppError;
use common_services::api::jobs::interfaces::{JobsQuery, PaginatedJobsResponse};
use common_services::api::jobs::service::{cancel_job, get_job_overview, retry_job};
use tracing::instrument;

#[instrument(skip(context), err(Debug))]
pub async fn job_summary_handler(
    State(context): State<ApiContext>,
    Query(query): Query<JobsQuery>,
) -> Result<Json<PaginatedJobsResponse>, AppError> {
    let overview = get_job_overview(&context.pool, query).await?;
    Ok(Json(overview))
}

#[instrument(skip(context), err(Debug))]
pub async fn cancel_job_handler(
    State(context): State<ApiContext>,
    Path(job_id): Path<i64>,
) -> Result<Json<()>, AppError> {
    cancel_job(&context.pool, job_id).await?;
    Ok(Json(()))
}

#[instrument(skip(context), err(Debug))]
pub async fn retry_job_handler(
    State(context): State<ApiContext>,
    Path(job_id): Path<i64>,
) -> Result<Json<()>, AppError> {
    retry_job(&context.pool, job_id).await?;
    Ok(Json(()))
}