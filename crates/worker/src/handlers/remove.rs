use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use common_photos::{Job, media_dir, thumbnails_dir};

/// Handles the removal of a media item from the database and filesystem.
///
/// # Errors
///
/// This function will return an error if the job is missing a `relative_path`,
/// or if any database or filesystem operations fail.
pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(relative_path) = &job.relative_path else {
        return Err(eyre!("Remove job has no associated relative_path"));
    };
    let file_path = media_dir().join(relative_path);

    // 1. Begin a transaction
    let mut tx = context.pool.begin().await?;

    // 2. Delete from the database and get the ID for filesystem cleanup.
    // Use the transaction object `tx` here.
    let deleted_id: Option<String> = sqlx::query_scalar!(
        "DELETE FROM media_item WHERE relative_path = $1 RETURNING id",
        &relative_path
    )
    .fetch_optional(&mut *tx)
    .await?;

    // 3. Commit the transaction. If this succeeds, the DB change is permanent.
    tx.commit().await?;

    // 4. Perform filesystem operations.
    // If these fail, the job will be marked as failed and can be retried.
    if let Some(id) = deleted_id {
        let thumb_dir = thumbnails_dir().join(id);
        if thumb_dir.exists() {
            tokio::fs::remove_dir_all(thumb_dir).await?;
        }
    }

    if file_path.exists() {
        tokio::fs::remove_file(file_path).await?;
    }

    Ok(JobResult::Done)
}
