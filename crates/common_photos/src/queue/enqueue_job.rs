use crate::{JobType, is_video_file, media_dir};
use color_eyre::eyre::Result;
use sqlx::{PgConnection, PgPool, PgTransaction};
use tracing::info;

/// Enqueues a full ingest and analysis job for a given file.
///
/// # Errors
///
/// Returns an error if any of the database operations fail.
pub async fn enqueue_full_ingest(pool: &PgPool, relative_path: &str, user_id: i32) -> Result<()> {
    enqueue_file_job(pool, JobType::Ingest, relative_path, user_id).await?;
    enqueue_file_job(pool, JobType::Analysis, relative_path, user_id).await?;
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

/// Enqueues a system-level job that isn't associated with a specific file.
///
/// # Errors
///
/// Returns an error if the database transaction fails.
pub async fn enqueue_system_job(pool: &PgPool, job_type: JobType) -> Result<()> {
    let priority = job_type.get_priority(false);
    let mut tx = pool.begin().await?;
    base_enqueue(&mut tx, job_type, None, None, priority).await?;
    tx.commit().await?;
    Ok(())
}

/// Enqueues a job for a specific file, such as ingestion or removal.
///
/// # Errors
///
/// Returns an error if the database transaction fails.
pub async fn enqueue_file_job(
    pool: &PgPool,
    job_type: JobType,
    relative_path: &str,
    user_id: i32,
) -> Result<()> {
    let is_video = is_video_file(&media_dir().join(relative_path));
    let priority = job_type.get_priority(is_video);

    let mut tx = pool.begin().await?;
    if job_type == JobType::Remove {
        prepare_remove_job(&mut tx, relative_path).await?;
    }
    base_enqueue(
        &mut tx,
        job_type,
        Some(relative_path),
        Some(user_id),
        priority,
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Handles the core logic of inserting a new job into the database if a similar one isn't already active.
///
/// # Errors
///
/// Returns an error if any of the database queries fail.
async fn base_enqueue(
    tx: &mut PgConnection,
    job_type: JobType,
    relative_path: Option<&str>,
    user_id: Option<i32>,
    priority: i32,
) -> Result<bool> {
    let job_exists = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM jobs
        WHERE relative_path = $1 AND job_type = $2 AND status IN ('running', 'queued', 'failed')
        LIMIT 1
        "#,
        relative_path,
        job_type as JobType
    )
    .fetch_optional(&mut *tx)
    .await?;

    if job_exists.is_some() {
        info!(
            "Not enqueueing {:?} job {:?}, it already exists.",
            job_type, relative_path
        );
        return Ok(false);
    }
    info!("Enqueueing {:?} job {:?}", job_type, relative_path);

    let result = sqlx::query!(
        r#"
        INSERT INTO jobs (relative_path, job_type, priority, user_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#,
        relative_path,
        job_type as JobType,
        priority,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    Ok(result.rows_affected() > 0)
}
