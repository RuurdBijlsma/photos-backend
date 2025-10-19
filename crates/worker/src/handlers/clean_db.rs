use crate::context::WorkerContext;
use crate::handlers::JobResult;
use chrono::Utc;
use color_eyre::Result;
use common_photos::Job;
use std::time::Duration;

pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    sqlx::query!(
        "DELETE FROM refresh_token WHERE expires_at < $1",
        Utc::now() - Duration::from_secs(1 * 60 * 60)
    )
    .execute(&context.pool)
    .await?;

    Ok(JobResult::Done)
}
