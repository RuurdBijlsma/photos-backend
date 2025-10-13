use crate::JobType;
use crate::utils::worker_id;
use chrono::Duration;
use chrono::Utc;
use color_eyre::Result;
use common_photos::{Job, JobStatus, alert};
use sqlx::{PgPool, PgTransaction};
use tracing::warn;

pub async fn claim_next_job(pool: &PgPool) -> Result<Option<Job>> {
    let mut tx = pool.begin().await?;

    let job = sqlx::query_as!(
        Job,
        r#"
        WITH candidate AS (
            SELECT id
            FROM jobs
            WHERE status = 'queued' AND scheduled_at <= now()
            ORDER BY priority, scheduled_at, created_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE jobs
        SET status = 'running',
            owner = $1,
            started_at = now()
        WHERE id = (SELECT id FROM candidate)
        RETURNING id, relative_path, job_type AS "job_type!: JobType", priority, user_id, attempts, max_attempts, dependency_attempts
        "#,
        worker_id()
    )
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(job)
}

pub async fn mark_job_done(pool: &PgPool, job_id: i64) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'done', finished_at = now()
        WHERE id = $1
        "#,
        job_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_job_failed(pool: &PgPool, job_id: i64, last_error: &str) -> Result<()> {
    alert!("‼️ Marking job {} as failed: {}", job_id, last_error);
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'failed',
            finished_at = now(),
            last_error = $2,
            attempts = attempts + 1
        WHERE id = $1
        "#,
        job_id,
        last_error
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn reschedule_job(pool: &PgPool, job_id: i64, backoff_secs: i64, last_error: &str) -> Result<()> {
    warn!("⚠️ Rescheduling job. Backoff: {:?}, last_err: {last_error}", backoff_secs);
    let scheduled_at = Utc::now() + Duration::seconds(backoff_secs);
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'queued',
            scheduled_at = $2,
            attempts = attempts + 1,
            owner = NULL,
            started_at = NULL,
            last_error = $3
        WHERE id = $1
        "#,
        job_id,
        scheduled_at,
        last_error
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn dependency_reschedule_job(
    pool: &PgPool,
    job_id: i64,
    backoff_secs: i64,
) -> Result<()> {
    warn!("Dependency check failed, reschedule job. Backoff: {:?}", backoff_secs);
    let scheduled_at = Utc::now() + Duration::seconds(backoff_secs);
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'queued',
            scheduled_at = $2,
            dependency_attempts = dependency_attempts + 1,
            owner = NULL,
            started_at = NULL,
            last_error = NULL
        WHERE id = $1
        "#,
        job_id,
        scheduled_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn is_job_cancelled(tx: &mut PgTransaction<'_>, job_id: i64) -> Result<bool> {
    let status: JobStatus = sqlx::query_scalar!(
        r#"
        SELECT status AS "status: JobStatus"
        FROM jobs
        WHERE id = $1
        "#,
        job_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(status == JobStatus::Cancelled)
}
