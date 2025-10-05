use crate::process_file::process_file;
use crate::WorkResult;
use color_eyre::eyre::bail;
use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use photos_core::{get_thumbnail_options, get_thumbnails_dir, worker_config};
use sqlx::{FromRow, PgPool, PgTransaction, Type};
use std::path::Path;

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
    let config = worker_config();
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
        JobType::Ingest => handle_ingest(&file, analyzer, &mut tx).await,
        JobType::Remove => handle_remove(&job, &file, &mut tx).await,
    };

    match job_result {
        Ok(_) => {
            // SUCCESS: Delete the job from the queue
            println!("✅ {:?}", &job);
            sqlx::query("DELETE FROM job_queue WHERE relative_path = $1")
                .bind(job.relative_path)
                .execute(&mut *tx)
                .await?;
        }
        Err(e) => {
            // FAILURE: Decide whether to retry or move to dead-letter queue
            let current_retries = job.retry_count.unwrap_or(0);

            if current_retries >= config.max_retries - 1 {
                println!("‼️ Moving to failures queue ‼️ {:?}", &job);
                // TODO alert here

                sqlx::query("INSERT INTO queue_failures (relative_path) VALUES ($1) ON CONFLICT (relative_path) DO NOTHING")
                    .bind(relative_path)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DELETE FROM job_queue WHERE relative_path = $1")
                    .bind(relative_path)
                    .execute(&mut *tx)
                    .await?;
            } else {
                println!("⚠️ Job {:?} {}", &job, e);
                sqlx::query("UPDATE job_queue SET retry_count = $1 WHERE relative_path = $2")
                    .bind(current_retries + 1)
                    .bind(relative_path)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(WorkResult::Processed)
}

async fn handle_ingest(
    file: &Path,
    analyzer: &mut MediaAnalyzer,
    tx: &mut PgTransaction<'_>,
) -> Result<()> {
    let thumb_dir = get_thumbnails_dir();
    let thumb_config = get_thumbnail_options(&thumb_dir);
    process_file(file, &thumb_config, analyzer, tx).await
}

async fn handle_remove(job: &Job, file: &Path, tx: &mut PgTransaction<'_>) -> Result<()> {
    // 1. Delete from main media items table (cascades should handle the rest)
    sqlx::query("DELETE FROM media_item WHERE relative_path = $1")
        .bind(&job.relative_path)
        .execute(&mut **tx)
        .await?;

    // 2. Delete thumbnails from the filesystem
    let thumb_dir = get_thumbnails_dir();
    let Some(filename) = file.file_name().map(|f| f.to_string_lossy().to_string()) else {
        bail!("Could not get filename for {:?}", file);
    };
    let thumb_file_dir = thumb_dir.join(filename);
    tokio::fs::remove_dir_all(thumb_file_dir).await?;

    Ok(())
}
