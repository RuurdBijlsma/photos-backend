#![allow(clippy::unnecessary_debug_formatting, clippy::cognitive_complexity)]

use crate::utils::{backoff_seconds, worker_id};
use color_eyre::Result;
use common_photos::{alert, file_is_ingested, get_db_pool, media_dir, Job, JobType};
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
    start_heartbeat_loop,
};
use ml_analysis::VisualAnalyzer;
use tokio::time::sleep;
use crate::JobHandleResult::{DependencyReschedule, Done};

mod db_helpers;
mod handlers;
mod jobs;
mod utils;

pub enum JobHandleResult {
    DependencyReschedule,
    Done,
}

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
pub async fn worker_loop(pool: &PgPool) -> Result<()> {
    let mut sleeping = false;
    let mut media_analyzer = MediaAnalyzer::builder().build().await?;
    let visual_analyzer = VisualAnalyzer::new()?;
    loop {
        if let Some(job) = claim_next_job(pool).await? {
            sleeping = false;
            info!("🐜 Picked up {:?} job: {}", job.job_type, job.relative_path);
            let job_id = job.id;

            // Spawn a background task to send heartbeats every 2 minutes
            let heartbeat_handle = start_heartbeat_loop(&pool, job_id);
            let result = match job.job_type {
                JobType::Ingest => ingest_file(pool, &job, &mut media_analyzer)
                    .await
                    .map(|_| Done),
                JobType::Remove => remove_file(pool, &job).await.map(|_| Done),
                JobType::Analysis => handle_analysis(pool, &job, &visual_analyzer).await,
            };
            heartbeat_handle.abort();

            match result {
                Ok(Done) => mark_job_done(pool, job.id).await?,
                Ok(DependencyReschedule)=>{
                    // ingest not ready → reschedule job
                    if job.dependency_attempts > 10 {
                        alert!("Alarmingly many attempts to dependency reschedule job.");
                    }
                    let delay = backoff_seconds(job.dependency_attempts);
                    dependency_reschedule_job(pool, job.id, delay).await?;
                }
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

async fn handle_analysis(
    pool: &PgPool,
    job: &Job,
    visual_analyzer: &VisualAnalyzer,
) -> Result<JobHandleResult> {
    let file_path = media_dir().join(&job.relative_path);
    if !file_is_ingested(&file_path, pool).await? {
        info!("file {} is not ingested properly", &job.relative_path);
        return Ok(DependencyReschedule);
    }
    analyze_file(pool, job, visual_analyzer).await?;
    Ok(Done)
}
