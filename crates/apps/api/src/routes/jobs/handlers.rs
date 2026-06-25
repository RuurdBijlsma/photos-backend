use crate::api_state::ApiContext;
use axum::Json;
use axum::extract::State;
use common_services::api::app_error::AppError;
use common_services::api::jobs::interfaces::JobsResponse;
use common_services::api::jobs::service::get_job_overview;
use tracing::instrument;

#[instrument(skip(context), err(Debug))]
pub async fn job_summary_handler(
    State(context): State<ApiContext>,
) -> Result<Json<JobsResponse>, AppError> {
    let overview = get_job_overview(&context.pool).await?;
    Ok(Json(overview))
}
