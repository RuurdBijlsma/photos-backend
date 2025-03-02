use anyhow::Error;
use std::time::Duration;
use tracing::info;

pub trait JobStatus {
    fn is_done(&self) -> bool;
}
pub async fn poll_job<J: JobStatus + serde::de::DeserializeOwned>(
    client: &crate::common::api_client::ApiClient,
    job_id: &str,
    delay_secs: u64,
    timeout_secs: u64,
    max_retries: u64,
    retry_delay_secs: u64,
) -> Result<J, loco_rs::Error> {
    let mut attempts = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;

        let mut retries = 0;
        let status: J = loop {
            match client.check_status(job_id).await {
                Ok(status) => break status,
                Err(e) => {
                    if retries >= max_retries {
                        return Err(loco_rs::Error::Message(format!(
                            "Max retries exceeded for job {}: {}",
                            job_id, e
                        )));
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
            return Err(loco_rs::Error::Message(format!(
                "Timeout waiting for job {}",
                job_id
            )));
        }
    }
}
