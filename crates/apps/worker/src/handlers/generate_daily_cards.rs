use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::Result;
use common_services::database::jobs::Job;

#[allow(clippy::unused_async)]
pub async fn handle(_context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    Ok(JobResult::Done)
}
