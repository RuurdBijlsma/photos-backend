use crate::WorkResult;
use crate::ingest_file::ingest_file;
use crate::remove_file::remove_file;
use color_eyre::{Report, Result};
use common_photos::settings;
use media_analyzer::MediaAnalyzer;
use sqlx::{FromRow, PgPool, PgTransaction, Type};
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Type, PartialEq, Eq)]
#[sqlx(type_name = "job_type", rename_all = "UPPERCASE")]
pub enum JobType {
    Ingest,
    Remove,
}

#[derive(FromRow, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Job {
    pub relative_path: String,
    pub retry_count: Option<i32>,
    pub job_type: JobType,
}

/// Fetches and processes a single job from the queue within a database transaction.
///
/// This function coordinates the full lifecycle of a job, delegating the core logic
/// to helper functions for execution, success, and failure handling.
///
/// # Errors
///
/// * Returns an `sqlx::Error` for any database transaction or query failures.
/// * Propagates an error from a job's execution if the job is eligible for a retry.
pub async fn process_one_job(
    media_dir: &Path,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> Result<WorkResult> {
    let mut tx = pool.begin().await?;

    let job: Option<Job> = sqlx::query_as(
        "
            SELECT relative_path, retry_count, job_type
            FROM job_queue
            ORDER BY priority, created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED",
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(job) = job else {
        return Ok(WorkResult::QueueEmpty);
    };

    let file = media_dir.join(&job.relative_path);
    let job_result = match job.job_type {
        JobType::Ingest => ingest_file(&file, analyzer, &mut tx).await,
        JobType::Remove => remove_file(&job, &file, &mut tx).await,
    };

    match job_result {
        Ok(()) => handle_job_success(&job, &mut tx).await?,
        Err(e) => {
            // If handling the failure indicates a retry, commit the state
            // and propagate the original error to the worker loop.
            if handle_job_failure(&job, &e, &mut tx).await? {
                tx.commit().await?;
                return Err(e);
            }
        }
    }

    tx.commit().await?;
    Ok(WorkResult::Processed)
}

/// Handles a successful job by logging it and deleting it from the queue.
async fn handle_job_success(job: &Job, tx: &mut PgTransaction<'_>) -> Result<()> {
    info!("✅ {:?}", &job);
    sqlx::query("DELETE FROM job_queue WHERE relative_path = $1")
        .bind(&job.relative_path)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Handles a failed job by either updating its retry count or moving it to the failures queue.
/// Returns `true` if the error should be propagated for a retry, `false` otherwise.
async fn handle_job_failure(job: &Job, e: &Report, tx: &mut PgTransaction<'_>) -> Result<bool> {
    let worker_config = &settings().worker;
    let current_retries = job.retry_count.unwrap_or(0);

    if current_retries >= worker_config.max_retries - 1 {
        warn!("‼️ Moving to failures queue ‼️ {:?}", &job);
        sqlx::query(
            "INSERT INTO queue_failures (relative_path, job_type)
             VALUES ($1, $2)
             ON CONFLICT (relative_path, job_type) DO NOTHING",
        )
        .bind(&job.relative_path)
        .bind(&job.job_type)
        .execute(&mut **tx)
        .await?;

        sqlx::query("DELETE FROM job_queue WHERE relative_path = $1")
            .bind(&job.relative_path)
            .execute(&mut **tx)
            .await?;

        Ok(false)
    } else {
        warn!("⚠️ {:?} {}", &job, e);
        sqlx::query!(
            "UPDATE job_queue SET retry_count = $1 WHERE relative_path = $2",
            current_retries + 1,
            &job.relative_path
        )
        .execute(&mut **tx)
        .await?;

        Ok(true)
    }
}
