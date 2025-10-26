use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::macros::backoff_seconds;
use chrono::{Duration, Utc};
use color_eyre::Result;
use common_photos::{Job, alert};
use common_photos::{JobStatus, JobType};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{info, warn};

/// Atomically claims the next available job from the queue.
///
/// # Errors
///
/// Returns an error if the database transaction fails.
pub async fn claim_next_job(context: &WorkerContext) -> Result<Option<Job>> {
    let mut tx = context.pool.begin().await?;
    let heartbeat_timeout_seconds = 300.;

    let job = sqlx::query_as!(
        Job,
        r#"
        WITH candidate AS (
            SELECT id FROM jobs
            WHERE ((status = 'queued' AND scheduled_at <= now())
               OR (status = 'running' AND last_heartbeat < now() - interval '1 second' * $2))
              AND ($3 OR job_type != 'analysis')
            ORDER BY priority, relative_path DESC, scheduled_at, created_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE jobs
        SET status = 'running',
            owner = $1,
            started_at = now(),
            last_heartbeat = now(),
            attempts = CASE WHEN status = 'running' THEN attempts + 1 ELSE attempts END
        WHERE id = (SELECT id FROM candidate)
        RETURNING id, relative_path, job_type AS "job_type!: JobType", priority, user_id, attempts, max_attempts, dependency_attempts
        "#,
        context.worker_id,
        heartbeat_timeout_seconds,
        context.handle_analysis
    )
        .fetch_optional(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(job)
}

/// Updates a job's status based on a successful completion result.
///
/// # Errors
///
/// Returns an error if the database update fails.
pub async fn update_job_on_completion(pool: &PgPool, job: &Job, result: JobResult) -> Result<()> {
    match result {
        JobResult::Done => mark_job_done(pool, job.id).await,
        JobResult::Cancelled => mark_job_cancelled(pool, job.id).await,
        JobResult::DependencyReschedule => {
            if job.dependency_attempts > 10 {
                alert!(
                    "Alarmingly many attempts to dependency reschedule job {}.",
                    job.id
                );
            }
            let delay = backoff_seconds(job.dependency_attempts);
            dependency_reschedule_job(pool, job.id, delay).await
        }
    }
}

/// Updates a job's status on failure, either marking it as failed or rescheduling it.
///
/// # Errors
///
/// Returns an error if the database update fails.
pub async fn update_job_on_failure(pool: &PgPool, job: &Job, error: &str) -> Result<()> {
    if job.attempts + 1 >= job.max_attempts {
        mark_job_failed(pool, job.id, error).await
    } else {
        let delay = backoff_seconds(job.attempts);
        reschedule_for_retry(pool, job.id, delay, error).await
    }
}

/// Marks a job as done in the database.
///
/// # Errors
///
/// Returns an error if the database query fails.
async fn mark_job_done(pool: &PgPool, job_id: i64) -> Result<()> {
    sqlx::query!(
        "UPDATE jobs SET status = 'done', finished_at = now() WHERE id = $1",
        job_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Marks a job as cancelled in the database.
///
/// # Errors
///
/// Returns an error if the database query fails.
async fn mark_job_cancelled(pool: &PgPool, job_id: i64) -> Result<()> {
    sqlx::query!("UPDATE jobs SET status = 'cancelled' WHERE id = $1", job_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Marks a job as failed in the database.
///
/// # Errors
///
/// Returns an error if the database query fails.
async fn mark_job_failed(pool: &PgPool, job_id: i64, last_error: &str) -> Result<()> {
    alert!("‼️ Marking job {} as failed: {}", job_id, last_error);
    sqlx::query!(
        "UPDATE jobs SET status = 'failed', finished_at = now(), last_error = $2, attempts = attempts + 1 WHERE id = $1",
        job_id,
        last_error
    )
        .execute(pool)
        .await?;
    Ok(())
}

/// Reschedules a job to be tried again after a backoff period.
///
/// # Errors
///
/// Returns an error if the database query fails.
async fn reschedule_for_retry(
    pool: &PgPool,
    job_id: i64,
    backoff_secs: i64,
    last_error: &str,
) -> Result<()> {
    warn!(
        "⚠️ Rescheduling job {}. Backoff: {}s, error: {}",
        job_id, backoff_secs, last_error
    );
    let scheduled_at = Utc::now() + Duration::seconds(backoff_secs);
    sqlx::query!(
        "UPDATE jobs SET status = 'queued', scheduled_at = $2, attempts = attempts + 1, owner = NULL, started_at = NULL, last_error = $3 WHERE id = $1",
        job_id,
        scheduled_at,
        last_error
    )
        .execute(pool)
        .await?;
    Ok(())
}

/// Reschedules a job because its dependencies are not met.
///
/// # Errors
///
/// Returns an error if the database query fails.
async fn dependency_reschedule_job(pool: &PgPool, job_id: i64, backoff_secs: i64) -> Result<()> {
    info!(
        "⏳ Dependency not met for job {}. Rescheduling in {}s.",
        job_id, backoff_secs
    );
    let scheduled_at = Utc::now() + Duration::seconds(backoff_secs);
    sqlx::query!(
        "UPDATE jobs SET status = 'queued', scheduled_at = $2, dependency_attempts = dependency_attempts + 1, owner = NULL, started_at = NULL, last_error = NULL WHERE id = $1",
        job_id,
        scheduled_at
    )
        .execute(pool)
        .await?;
    Ok(())
}

/// Checks if a job has been cancelled within a given transaction.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn is_job_cancelled(
    transaction: &mut Transaction<'_, Postgres>,
    job_id: i64,
) -> Result<bool> {
    let status: Option<JobStatus> = sqlx::query_scalar("SELECT status FROM jobs WHERE id = $1")
        .bind(job_id)
        .fetch_optional(&mut **transaction)
        .await?;
    Ok(status.is_none_or(|s| s == JobStatus::Cancelled))
}
