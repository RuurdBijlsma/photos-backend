use crate::utils::{backoff_seconds, worker_id};
use color_eyre::Result;
use common_photos::{alert, file_is_ingested, get_db_pool, media_dir, settings, JobType};
use media_analyzer::MediaAnalyzer;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};

use crate::handlers::analyze_file::analyze_file;
use crate::handlers::ingest_file::ingest_file;
use crate::handlers::remove_file::remove_file;
use crate::jobs::{
    claim_next_job, increment_dependency_attempts, mark_job_done, mark_job_failed, reschedule_job,
};
use tokio::time::sleep;

mod db_helpers;
mod handlers;
mod jobs;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    info!("[Worker PID: {}] Starting.", worker_id());
    let mut analyzer = MediaAnalyzer::builder().build().await?;
    let worker_config = &settings().worker;
    let pool = get_db_pool().await?;

    worker_loop(&pool).await?;

    Ok(())
}

pub async fn worker_loop(pool: &PgPool) -> Result<()> {
    loop {
        if let Some(job) = claim_next_job(pool).await? {
            let result = match job.job_type {
                JobType::Ingest => ingest_file(pool, &job).await,
                JobType::Remove => remove_file(pool, &job).await,
                JobType::Analysis => {
                    let file_path = media_dir().join(&job.relative_path);
                    if !file_is_ingested(&file_path, pool).await? {
                        // ingest not ready → reschedule job
                        increment_dependency_attempts(pool, job.id).await?;
                        if job.dependency_attempts > 10 {
                            alert("Alarmingly many attempts to reschedule analysis job.");
                        }
                        let delay = backoff_seconds(job.dependency_attempts);
                        reschedule_job(pool, job.id, delay).await?;
                        continue;
                    }
                    analyze_file(pool, &job).await
                }
            };

            // 3. handle success/failure
            match result {
                Ok(()) => mark_job_done(pool, job.id).await?,
                Err(err) => {
                    if job.attempts + 1 >= job.max_attempts {
                        // give up
                        mark_job_failed(pool, job.id, &err.to_string()).await?;
                    } else {
                        // reschedule with exponential backoff
                        let delay = backoff_seconds(job.attempts);
                        reschedule_job(pool, job.id, delay).await?;
                    }
                }
            }
        } else {
            // no jobs ready → short sleep
            sleep(Duration::from_millis(500)).await;
        }
    }
}
