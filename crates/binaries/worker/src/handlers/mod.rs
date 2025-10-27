use crate::context::WorkerContext;
use crate::jobs::heartbeat::start_heartbeat_loop;
use color_eyre::Result;
use common_photos::{Job, JobType};

pub mod analyze;
pub mod clean_db;
pub mod ingest;
pub mod remove;
pub mod scan;

pub mod db;

/// The outcome of a job handler's execution.
#[derive(Debug, PartialEq, Eq)]
pub enum JobResult {
    Done,
    Cancelled,
    DependencyReschedule,
}

/// Dispatches a job to its corresponding handler and manages its lifecycle.
///
/// # Errors
///
/// This function will return an error if the specific job handler fails during execution.
pub async fn handle_job(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let heartbeat_handle = start_heartbeat_loop(&context.pool, job.id);

    let result = match job.job_type {
        JobType::Ingest => ingest::handle(context, job).await,
        JobType::Analysis => analyze::handle(context, job).await,
        JobType::Remove => remove::handle(context, job).await,
        JobType::Scan => scan::handle(context, job).await,
        JobType::CleanDB => clean_db::handle(context, job).await,
    };

    heartbeat_handle.abort();
    result
}
