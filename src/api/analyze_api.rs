use crate::api::analyze_structs::{MediaAnalyzerOutput, ProcessingJob, ProcessingRequest};
use crate::common::api_client::ApiClient;
use crate::common::job_polling::{poll_job, JobStatus};
use crate::common::settings::Settings;

impl JobStatus for ProcessingJob {
    fn is_done(&self) -> bool {
        self.done
    }
}

pub async fn process_media(
    relative_path: String,
    settings: &Settings,
) -> Result<MediaAnalyzerOutput, loco_rs::Error> {
    let client = ApiClient::new(&settings.processing_api_url, "process");
    let request = ProcessingRequest { relative_path };

    let job_id = client
        .submit_job(&request)
        .await
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;

    let status: ProcessingJob = poll_job(
        &client, &job_id, 5,   // delay_secs
        300, // timeout_secs (5 minutes)
        3,   // max_retries
        1,   // retry_delay_secs
    )
    .await
    .map_err(|e| loco_rs::Error::Message(e.to_string()))?;

    if let Some(result) = status.result {
        client
            .delete_job(&job_id)
            .await
            .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
        Ok(result)
    } else {
        Err(loco_rs::Error::Message(
            "Processing completed but no result available".to_string(),
        ))
    }
}
