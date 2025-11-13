use crate::context::WorkerContext;
use crate::handlers::JobResult;
use chrono::Utc;
use color_eyre::Result;
use common_services::queue::Job;
use std::time::Duration;

/// Deletes expired refresh tokens from the database.
///
/// # Errors
///
/// This function will return an error if the database query fails.
pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    sqlx::query!(
        "DELETE FROM refresh_token WHERE expires_at < $1",
        Utc::now() - Duration::from_secs(1 * 60 * 60)
    )
    .execute(&context.pool)
    .await?;

    Ok(JobResult::Done)
}
