#![allow(clippy::unnecessary_debug_formatting)]

use crate::utils::{backoff_seconds, worker_id};
use color_eyre::Result;
use common_photos::{alert, file_is_ingested, get_db_pool, media_dir, JobType};
use media_analyzer::MediaAnalyzer;
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;
use tracing::warn;

use crate::handlers::analyze_file::analyze_file;
use crate::handlers::ingest_file::ingest_file;
use crate::handlers::remove_file::remove_file;
use crate::jobs::{
    claim_next_job, dependency_reschedule_job, mark_job_done, mark_job_failed, reschedule_job,
};
use ml_analysis::VisualAnalyzer;
use tokio::time::sleep;

mod db_helpers;
mod handlers;
mod jobs;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    info!("[Worker ID: {}] Starting.", worker_id());
    let pool = get_db_pool().await?;

    worker_loop(&pool).await?;

    Ok(())
}

/// The main loop for the worker process, continuously fetching and processing jobs.
/// # Errors
///
/// * Returns an error if building the `MediaAnalyzer` fails.
/// * Returns an error if there's a problem claiming a job from the database.
/// * Propagates errors from job handlers or database updates that are unrecoverable.
#[allow(clippy::cognitive_complexity)]
pub async fn worker_loop(pool: &PgPool) -> Result<()> {
    let mut sleeping = false;
    let mut media_analyzer = MediaAnalyzer::builder().build().await?;
    let visual_analyzer = VisualAnalyzer::new()?;
    loop {
        if let Some(job) = claim_next_job(pool).await? {
            sleeping = false;
            info!("🐜 Picked up {:?} job: {}", job.job_type, job.relative_path);
            let result = match job.job_type {
                JobType::Ingest => ingest_file(pool, &job, &mut media_analyzer).await,
                JobType::Remove => remove_file(pool, &job).await,
                JobType::Analysis => {
                    let file_path = media_dir().join(&job.relative_path);
                    if !file_is_ingested(&file_path, pool).await? {
                        info!("file {} is not ingested properly", &file_path.display());
                        // ingest not ready → reschedule job
                        if job.dependency_attempts > 10 {
                            alert!("Alarmingly many attempts to reschedule analysis job.");
                        }
                        let delay = backoff_seconds(job.dependency_attempts);
                        dependency_reschedule_job(pool, job.id, delay).await?;
                        continue;
                    }
                    analyze_file(pool, &job, &visual_analyzer).await
                }
            };

            // 3. handle success/failure
            match result {
                Ok(()) => mark_job_done(pool, job.id).await?,
                Err(err) => {
                    if job.attempts + 1 >= job.max_attempts {
                        mark_job_failed(pool, job.id, &err.to_string()).await?;
                    } else {
                        let delay = backoff_seconds(job.attempts);
                        reschedule_job(pool, job.id, delay, &err.to_string()).await?;
                    }
                }
            }
        } else {
            if !sleeping {
                sleeping = true;
                info!("💤 No jobs, sleeping a bit...");
            }
            sleep(Duration::from_millis(3000)).await;
        }
    }
}
