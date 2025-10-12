use crate::WorkResult;
use crate::ingest_file::ingest_file;
use crate::ml_analyze_file::ml_analyze_file;
use crate::remove_file::remove_file;
use color_eyre::{Report, Result};
use common_photos::{enqueue_analysis, settings};
use media_analyzer::MediaAnalyzer;
use sqlx::{FromRow, PgPool, PgTransaction, Type};
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Type, PartialEq, Eq, Clone)]
#[sqlx(type_name = "job_type", rename_all = "UPPERCASE")]
pub enum JobType {
    Ingest,
    Remove,
    Analysis,
}

#[derive(FromRow, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Job {
    pub id: i32,
    pub relative_path: String,
    pub retry_count: Option<i32>,
    pub job_type: JobType,
    pub user_id: i32,
}

pub async fn process_one_job(
    media_dir: &Path,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> Result<WorkResult> {
    let mut tx = pool.begin().await?;

    // Fetch the job's 'id' as well.
    let job: Option<Job> = sqlx::query_as(
        "
            SELECT id, relative_path, retry_count, job_type, user_id
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
        JobType::Ingest => ingest_file(&file, job.user_id, analyzer, &mut tx).await,
        JobType::Remove => remove_file(&job, &file, &mut tx).await,
        JobType::Analysis => ml_analyze_file(&file, analyzer, &mut tx).await, // Handle new job
    };

    match job_result {
        Ok(()) => handle_job_success(&job, pool, &mut tx).await?,
        Err(e) => {
            if handle_job_failure(&job, &e, &mut tx).await? {
                tx.commit().await?;
                return Err(e);
            }
        }
    }

    tx.commit().await?;
    Ok(WorkResult::Processed)
}

/// Handles a successful job by deleting it from the queue using its unique ID.
async fn handle_job_success(job: &Job, pool: &PgPool, tx: &mut PgTransaction<'_>) -> Result<()> {
    info!("✅ {:?} ({:?})", &job.job_type, &job.relative_path);
    sqlx::query("DELETE FROM job_queue WHERE id = $1")
        .bind(job.id)
        .execute(&mut **tx)
        .await?;

    // If the completed job was an INGEST job, enqueue the ANALYSIS job.
    if job.job_type == JobType::Ingest {
        enqueue_analysis(&job.relative_path, job.user_id, pool).await?;
    }
    Ok(())
}

/// Handles a failed job, using the job's unique ID for updates/deletions.
async fn handle_job_failure(job: &Job, e: &Report, tx: &mut PgTransaction<'_>) -> Result<bool> {
    let worker_config = &settings().worker;
    let current_retries = job.retry_count.unwrap_or(0);

    if current_retries >= worker_config.max_retries - 1 {
        warn!(
            "‼️ Moving to failures queue ‼️ {:?} ({:?})",
            &job.job_type, &job.relative_path
        );
        sqlx::query(
            "INSERT INTO queue_failures (relative_path, user_id, job_type)
             VALUES ($1, $2, $3)
             ON CONFLICT (relative_path, job_type) DO NOTHING",
        )
        .bind(&job.relative_path)
        .bind(job.user_id)
        .bind(&job.job_type)
        .execute(&mut **tx)
        .await?;

        // Use the primary key for deletion.
        sqlx::query("DELETE FROM job_queue WHERE id = $1")
            .bind(job.id)
            .execute(&mut **tx)
            .await?;

        Ok(false)
    } else {
        warn!("⚠️ {:?} ({:?}): {}", &job.job_type, &job.relative_path, e);
        // Use the primary key for the update.
        sqlx::query!(
            "UPDATE job_queue SET retry_count = $1 WHERE id = $2",
            current_retries + 1,
            job.id
        )
        .execute(&mut **tx)
        .await?;

        Ok(true)
    }
}
