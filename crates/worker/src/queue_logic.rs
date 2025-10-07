use crate::WorkResult;
use crate::ingest_file::ingest_file;
use crate::remove_file::remove_file;
use color_eyre::Result;
use common_photos::get_config;
use media_analyzer::MediaAnalyzer;
use sqlx::{FromRow, PgPool, Type};
use std::path::Path;
use tracing::{info, warn};

#[derive(Debug, Type, PartialEq)]
#[sqlx(type_name = "job_type", rename_all = "UPPERCASE")]
pub enum JobType {
    Ingest,
    Remove,
}

#[derive(FromRow, Debug)]
pub struct Job {
    pub relative_path: String,
    pub retry_count: Option<i32>,
    pub job_type: JobType,
}

pub async fn process_one_job(
    media_dir: &Path,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> Result<WorkResult> {
    let config = &get_config().worker;
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

    let relative_path = &job.relative_path;
    let file = media_dir.join(relative_path);

    let job_result = match job.job_type {
        JobType::Ingest => ingest_file(&file, analyzer, &mut tx).await,
        JobType::Remove => remove_file(&job, &file, &mut tx).await,
    };

    match job_result {
        Ok(_) => {
            // SUCCESS: Delete the job from the queue
            info!("✅ {:?}", &job);
            sqlx::query("DELETE FROM job_queue WHERE relative_path = $1")
                .bind(job.relative_path)
                .execute(&mut *tx)
                .await?;
        }
        Err(e) => {
            // FAILURE: Decide whether to retry or move to dead-letter queue
            let current_retries = job.retry_count.unwrap_or(0);

            if current_retries >= config.max_retries - 1 {
                warn!("‼️ Moving to failures queue ‼️ {:?}", &job);
                // TODO alert here

                sqlx::query(
                    "INSERT INTO queue_failures (relative_path, job_type)
                                    VALUES ($1, $2)
                                    ON CONFLICT (relative_path, job_type) DO NOTHING",
                )
                .bind(relative_path)
                .bind(job.job_type)
                .execute(&mut *tx)
                .await?;
                sqlx::query("DELETE FROM job_queue WHERE relative_path = $1")
                    .bind(relative_path)
                    .execute(&mut *tx)
                    .await?;
            } else {
                warn!("⚠️ {:?} {}", &job, e);
                sqlx::query!(
                    "UPDATE job_queue SET retry_count = $1 WHERE relative_path = $2",
                    current_retries + 1,
                    relative_path
                )
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                return Err(e);
            }
        }
    }

    tx.commit().await?;
    Ok(WorkResult::Processed)
}
