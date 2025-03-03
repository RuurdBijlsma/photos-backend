use crate::common::api_client::{ApiClient, ApiClientError};
use std::time::Duration;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum JobPollingError {
    #[error("Max retries exceeded for job {job_id}: {cause}")]
    MaxRetriesExceeded { job_id: String, cause: String },
    #[error("Timeout waiting for job {job_id}")]
    Timeout { job_id: String },
    #[error("API error: {0}")]
    Api(#[from] ApiClientError),
}

pub trait JobStatus {
    fn is_done(&self) -> bool;
}

/// Poll job running on api server.
///
/// # Errors
/// * Max retries exceeded when checking status
/// * Timeout if job takes too long.
pub async fn poll_job<J: JobStatus + serde::de::DeserializeOwned>(
    client: &ApiClient,
    job_id: &str,
    delay_secs: u64,
    timeout_secs: u64,
    max_retries: u64,
    retry_delay_secs: u64,
) -> Result<J, JobPollingError> {
    let mut attempts = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;

        let mut retries = 0;
        let status: J = loop {
            match client.check_status(job_id).await {
                Ok(status) => break status,
                Err(e) => {
                    if retries >= max_retries {
                        return Err(JobPollingError::MaxRetriesExceeded {
                            job_id: job_id.to_string(),
                            cause: e.to_string(),
                        });
                    }
                    retries += 1;
                    let sleep_duration = Duration::from_secs(retry_delay_secs * retries);
                    tokio::time::sleep(sleep_duration).await;
                    info!(
                        "Retrying check_status for job {} (attempt {}/{})",
                        job_id, retries, max_retries
                    );
                }
            }
        };

        if status.is_done() {
            return Ok(status);
        }

        attempts += 1;
        if attempts * delay_secs >= timeout_secs {
            return Err(JobPollingError::Timeout {
                job_id: job_id.to_string(),
            });
        }
    }
}
