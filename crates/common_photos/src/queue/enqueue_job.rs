use crate::{JobType, is_video_file, media_dir, user_id_from_relative_path};
use color_eyre::eyre::Result;
use sqlx::{PgConnection, PgPool};
use tracing::info;

/// Enqueues a full ingest and analysis job for a given file.
///
/// # Errors
///
/// Returns an error if any of the database operations fail.
pub async fn enqueue_full_ingest(pool: &PgPool, relative_path: &str) -> Result<()> {
    enqueue_with_priority(pool, relative_path, JobType::Ingest).await?;
    enqueue_with_priority(pool, relative_path, JobType::Analysis).await?;
    Ok(())
}

/// Cancels any queued ingest or analysis jobs and enqueues a remove job for the given file.
///
/// # Errors
///
/// Returns an error if any of the database queries or the transaction commit fails.
pub async fn enqueue_remove_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE jobs SET status = 'cancelled' WHERE relative_path = $1 AND status = 'queued' AND job_type IN ('ingest', 'analysis')",
        relative_path
    )
        .execute(&mut *tx)
        .await?;

    enqueue_job(&mut tx, relative_path, JobType::Remove, 0).await?;

    tx.commit().await?;
    Ok(())
}

async fn enqueue_with_priority(
    pool: &PgPool,
    relative_path: &str,
    job_type: JobType,
) -> Result<()> {
    let is_video = is_video_file(&media_dir().join(relative_path));
    let priority = match job_type {
        JobType::Ingest => {
            if is_video {
                20
            } else {
                10
            }
        }
        JobType::Analysis => {
            if is_video {
                100
            } else {
                90
            }
        }
        JobType::Remove => 0,
    };

    let mut tx = pool.begin().await?;
    enqueue_job(&mut tx, relative_path, job_type, priority).await?;
    tx.commit().await?;
    Ok(())
}

async fn enqueue_job(
    tx: &mut PgConnection,
    relative_path: &str,
    job_type: JobType,
    priority: i32,
) -> Result<bool> {
    let user_id = user_id_from_relative_path(relative_path, &mut *tx).await?;

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
            "Not enqueueing {:?} job {}, it already exists.",
            job_type, relative_path
        );
        return Ok(false);
    }
    info!("Enqueueing {:?} job {}", job_type, relative_path);

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
