use crate::context::WorkerContext;
use crate::handlers::JobResult;
use chrono::Utc;
use color_eyre::Result;
use common_services::database::jobs::Job;
use std::time::Duration;

/// Deletes expired refresh tokens from the database.
pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    // Remove expired refresh tokens
    sqlx::query!(
        "DELETE FROM refresh_token WHERE expires_at < $1",
        Utc::now() - Duration::from_secs(1 * 60 * 60)
    )
    .execute(&context.pool)
    .await?;

    // Sync media_count field on albums in case it drifted for some reason
    sqlx::query!(
        r"UPDATE album a
        SET media_count = (
            SELECT COUNT(*)
            FROM album_media_item ami
            JOIN media_item mi ON ami.media_item_id = mi.id
            WHERE ami.album_id = a.id AND mi.deleted = false
        )"
    )
    .execute(&context.pool)
    .await?;

    Ok(JobResult::Done)
}
