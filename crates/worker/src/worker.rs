use crate::context::WorkerContext;
use crate::handlers::handle_job;
use crate::jobs::management::{claim_next_job, update_job_on_completion, update_job_on_failure};
use color_eyre::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

/// The main loop for the worker process, continuously fetching and processing jobs.
///
/// # Errors
///
/// This function will return an error if there is a problem communicating with the
/// database when claiming or updating a job. The loop will terminate in such a case.
pub async fn run_worker_loop(context: &WorkerContext) -> Result<()> {
    let mut sleeping = false;

    loop {
        let maybe_job = claim_next_job(context).await?;

        if let Some(job) = maybe_job {
            sleeping = false;
            info!(
                "🐜 Picked up {:?} job: {:?}",
                job.job_type, job.relative_path
            );

            let job_result = handle_job(context, &job).await;

            match job_result {
                Ok(result) => update_job_on_completion(&context.pool, &job, result).await?,
                Err(e) => update_job_on_failure(&context.pool, &job, &e.to_string()).await?,
            }
        } else {
            if !sleeping {
                sleeping = true;
                info!("💤 No jobs, going to sleep...");
            }
            sleep(Duration::from_millis(3000)).await;
        }
    }
}
