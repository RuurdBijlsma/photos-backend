use crate::context::WorkerContext;
use crate::jobs::heartbeat::start_heartbeat_loop;
use color_eyre::Result;
use common_photos::{Job, JobType};

mod analyze;
mod ingest;
mod remove;

pub mod db;

/// The outcome of a job handler's execution.
#[derive(Debug, PartialEq, Eq)]
pub enum JobResult {
    Done,
    Cancelled,
    DependencyReschedule,
}

/// Dispatches a job to its corresponding handler and manages its lifecycle.
pub async fn handle_job(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let heartbeat_handle = start_heartbeat_loop(&context.pool, job.id);

    let result = match job.job_type {
        JobType::Ingest => ingest::handle(context, job).await,
        JobType::Analysis => analyze::handle(context, job).await,
        JobType::Remove => remove::handle(context, job).await,
    };

    heartbeat_handle.abort();
    result
}
