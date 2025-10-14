use crate::utils::worker_id;
use crate::JobType;
use chrono::Duration;
use chrono::Utc;
use color_eyre::Result;
use common_photos::{alert, Job, JobStatus};
use sqlx::{PgPool, PgTransaction};
use tokio::task::JoinHandle;
use tracing::warn;

pub async fn claim_next_job(pool: &PgPool) -> Result<Option<Job>> {
    let mut tx = pool.begin().await?;
    // A worker is considered dead if it hasn't sent a heartbeat in 5 minutes.
    let heartbeat_timeout_seconds = 300.;

    let job = sqlx::query_as!(
        Job,
        r#"
        WITH candidate AS (
            SELECT id
            FROM jobs
            WHERE
                -- Standard case: a job is ready to be run
                (status = 'queued' AND scheduled_at <= now())
                OR
                -- Recovery case: a job was running but the worker missed its heartbeat
                (status = 'running' AND last_heartbeat < now() - interval '1 second' * $2)
            ORDER BY priority, scheduled_at, created_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE jobs
        SET status = 'running',
            owner = $1,
            started_at = now(),
            last_heartbeat = now(),
            -- Increment attempts for jobs that are being retried after a heartbeat timeout
            attempts = CASE
                WHEN status = 'running' THEN attempts + 1
                ELSE attempts
            END
        WHERE id = (SELECT id FROM candidate)
        RETURNING id, relative_path, job_type AS "job_type!: JobType", priority, user_id, attempts, max_attempts, dependency_attempts
        "#,
        worker_id(),
        heartbeat_timeout_seconds
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

pub async fn reschedule_job(
    pool: &PgPool,
    job_id: i64,
    backoff_secs: i64,
    last_error: &str,
) -> Result<()> {
    warn!(
        "⚠️ Rescheduling job. Backoff: {:?}, last_err: {last_error}",
        backoff_secs
    );
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
    warn!(
        "Dependency check failed, reschedule job. Backoff: {:?}",
        backoff_secs
    );
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

pub fn start_heartbeat_loop(pool: &PgPool, job_id: i64) -> JoinHandle<()> {
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));
        loop {
            interval.tick().await;
            let result = sqlx::query!(
                "UPDATE jobs SET last_heartbeat = now() WHERE id = $1 AND status = 'running'",
                job_id
            )
            .execute(&pool_clone)
            .await;

            // If the update fails or affects 0 rows (job is no longer 'running'), stop heartbeat.
            if result.is_err() || result.unwrap().rows_affected() == 0 {
                break;
            }
        }
    })
}
