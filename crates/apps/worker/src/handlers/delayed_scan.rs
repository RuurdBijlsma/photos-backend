use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::Result;
use common_services::database::jobs::{Job, JobType};
use common_services::job_queue::enqueue_job;

pub async fn handle(context: &WorkerContext, _job: &Job) -> Result<JobResult> {
    enqueue_job::<()>(&context.pool, &context.settings, JobType::Scan)
        .call()
        .await?;

    Ok(JobResult::Done)
}
