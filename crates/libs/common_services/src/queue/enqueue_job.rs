use crate::queue::JobType;
use crate::settings::media_dir;
use crate::utils::is_video_file;
use bon::builder;
use color_eyre::eyre::Result;
use serde::Serialize;
use serde_json::to_value;
use sqlx::{PgPool, PgTransaction};
use tracing::{info, warn};

/// Enqueues a job for a specific file, such as ingestion or removal.
///
/// # Errors
///
/// Returns an error if the database transaction fails.
#[builder]
pub async fn enqueue_job<T: Serialize + Send + Sync>(
    #[builder(start_fn)] pool: &PgPool,
    #[builder(start_fn)] job_type: JobType,
    #[builder(into)] relative_path: Option<String>,
    user_id: Option<i32>,
    payload: Option<&T>,
) -> Result<bool> {
    let json_payload = payload.and_then(|p| to_value(p).ok());

    let mut tx = pool.begin().await?;

    if job_type == JobType::Remove
        && let Some(path) = &relative_path
    {
        prepare_remove_job(&mut tx, path).await?;
    }

    let is_video = relative_path
        .as_ref()
        .is_some_and(|p| is_video_file(&media_dir().join(p)));
    let priority = job_type.get_priority(is_video);

    let job_exists = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM jobs
        WHERE relative_path = $1 AND job_type = $2 AND payload = $3 AND status IN ('running', 'queued', 'failed')
        LIMIT 1
        "#,
        relative_path.as_deref(),
        job_type as JobType,
        json_payload,
    )
        .fetch_optional(&mut *tx)
        .await?;

    if job_exists.is_some() {
        warn!(
            "Not enqueueing {:?} job {:?}, it already exists.",
            job_type, relative_path
        );
        return Ok(false);
    }
    info!("Enqueueing {:?} job {:?}", job_type, relative_path);

    let result = sqlx::query!(
        r#"
        INSERT INTO jobs (relative_path, job_type, priority, user_id, payload)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT DO NOTHING
        "#,
        relative_path.as_deref(),
        job_type as JobType,
        priority,
        user_id,
        json_payload,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

/// Enqueues a full ingest and analysis job for a given file.
///
/// # Errors
///
/// Returns an error if any of the database operations fail.
pub async fn enqueue_full_ingest(pool: &PgPool, relative_path: &str, user_id: i32) -> Result<()> {
    enqueue_job::<()>(pool, JobType::Ingest)
        .relative_path(relative_path)
        .user_id(user_id)
        .call()
        .await?;
    enqueue_job::<()>(pool, JobType::Analysis)
        .relative_path(relative_path)
        .user_id(user_id)
        .call()
        .await?;

    Ok(())
}

/// Cancels any queued ingest or analysis jobs.
///
/// # Errors
///
/// Returns an error if any of the database queries or the transaction commit fails.
async fn prepare_remove_job(tx: &mut PgTransaction<'_>, relative_path: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE jobs SET status = 'cancelled' WHERE relative_path = $1 AND status = 'queued' AND job_type IN ('ingest', 'analysis')",
        relative_path
    )
        .execute(&mut **tx)
        .await?;
    Ok(())
}
