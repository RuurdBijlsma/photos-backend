use crate::user_id_from_relative_path;
use crate::{JobType, is_video_file, media_dir};
use color_eyre::eyre::Result;
use sqlx::{PgConnection, PgPool};
use tracing::info;

/// Enqueues two jobs, one to ingest a media file, and the other to analyze it with ML.
/// # Errors
///
/// * Returns an error if the database transaction in either enqueue function fails.
pub async fn enqueue_full_ingest(pool: &PgPool, relative_path: &str) -> Result<()> {
    enqueue_ingest_job(pool, relative_path).await?;
    enqueue_analysis_job(pool, relative_path).await?;

    Ok(())
}

/// Enqueues a job to ingest a media file.
/// # Errors
///
/// * Returns an error if the database transaction fails.
async fn enqueue_ingest_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let is_video = is_video_file(&media_dir().join(relative_path));
    let priority = if is_video { 20 } else { 10 };

    let mut tx = pool.begin().await?;
    enqueue_job(&mut tx, relative_path, JobType::Ingest, priority).await?;
    tx.commit().await?;

    Ok(())
}

/// Enqueues a job to perform machine learning analysis on a media file.
/// # Errors
///
/// * Returns an error if the database transaction fails.
async fn enqueue_analysis_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let is_video = is_video_file(&media_dir().join(relative_path));
    let priority = if is_video { 100 } else { 90 };

    let mut tx = pool.begin().await?;
    enqueue_job(&mut tx, relative_path, JobType::Analysis, priority).await?;
    tx.commit().await?;

    Ok(())
}

/// Cancels any queued ingest or analysis jobs for a file and enqueues a new job to remove it.
/// # Errors
///
/// * Returns an error if the database transaction fails.
pub async fn enqueue_remove_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let mut tx = pool.begin().await?;

    // cancel queued ingest/analysis for same file
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'cancelled'
        WHERE relative_path = $1
          AND status = 'queued'
          AND job_type IN ('ingest', 'analysis')
        "#,
        relative_path
    )
    .execute(&mut *tx)
    .await?;

    // enqueue remove with the highest priority
    enqueue_job(&mut tx, relative_path, JobType::Remove, 0).await?;

    tx.commit().await?;
    Ok(())
}

/// Inserts a new job into the database within a given transaction.
/// # Errors
///
/// * Returns an error if the user ID cannot be determined from the relative path.
/// * Returns an error if the database INSERT query fails.
async fn enqueue_job(
    tx: &mut PgConnection,
    relative_path: &str,
    job_type: JobType,
    priority: i32,
) -> Result<()> {
    // todo: probably don't enqueue job if it's marked as failed?
    let user_id = user_id_from_relative_path(relative_path, &mut *tx).await?;

    let job_exists = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM jobs
        WHERE relative_path = $1 AND job_type = $2
        LIMIT 1
        "#,
        relative_path,
        job_type as JobType
    )
    .fetch_optional(&mut *tx)
    .await?;

    if job_exists.is_some() {
        info!(
            "Not enqueueing {:?} job {}, it already exists.",
            job_type, relative_path
        );
        return Ok(());
    }
    info!("Enqueueing {:?} job {}", job_type, relative_path);

    sqlx::query!(
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

    Ok(())
}
