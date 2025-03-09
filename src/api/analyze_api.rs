use crate::api::analyze_structs::{MediaAnalyzerOutput, ProcessingJob, ProcessingRequest};
use crate::common::api_client::{ApiClient, ApiClientError};
use crate::common::job_polling::{poll_job, JobPollingError, JobStatus};

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("API error: {0}")]
    Api(#[from] ApiClientError),
    #[error("Job polling error: {0}")]
    JobPolling(#[from] JobPollingError),
    #[error("Processing completed with no result")]
    NoResult,
}

impl JobStatus for ProcessingJob {
    fn is_done(&self) -> bool {
        self.done
    }
}

/// Analyze image by sending it to the analysis api.
///
/// # Errors
/// * When submit job fails.
/// * When poll job fails.
/// * When delete job fails.
/// * When job has no result.
pub async fn analyze_image(
    relative_path: String,
    processing_api_url: &str,
) -> Result<MediaAnalyzerOutput, AnalyzeError> {
    let client = ApiClient::new(processing_api_url, "process");
    let request = ProcessingRequest { relative_path };

    let job_id = client.submit_job(&request).await?;

    let status: ProcessingJob = poll_job(
        &client, &job_id, 5,   // delay_secs
        300, // timeout_secs
        3,   // max_retries
        1,   // retry_delay_secs
    )
    .await?;

    if let Some(result) = status.result {
        client.delete_job(&job_id).await?;
        Ok(result)
    } else {
        Err(AnalyzeError::NoResult)
    }
}
