use crate::common::settings::Settings;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use std::future::Future;
use tracing::{error, info, warn};

// -- Data structures --
#[derive(Debug, Serialize, Deserialize)]
struct ThumbnailRequest {
    photos: Vec<String>,
    videos: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailJob {
    photos_done: i32,
    photos_total: i32,
    videos_done: i32,
    videos_total: i32,
    done: bool,
}

// -- API Client --
struct ThumbnailClient {
    client: Client,
    base_url: String,
}

impl ThumbnailClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
        }
    }

    pub async fn submit_job(
        &self,
        request: &ThumbnailRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/thumbnails", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {status}: {text}").into())
            }
        }
    }

    pub async fn check_status(
        &self,
        job_id: &str,
    ) -> Result<ThumbnailJob, Box<dyn std::error::Error>> {
        let url = format!("{}/thumbnails/{}", self.base_url, job_id);
        let response = self.client.get(&url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {status}: {text}").into())
            }
        }
    }

    pub async fn delete_job(&self, job_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/thumbnails/{}", self.base_url, job_id);
        let response = self.client.delete(&url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {status}: {text}").into())
            }
        }
    }
}

fn split_media_paths(paths: Vec<String>) -> (Vec<String>, Vec<String>) {
    paths.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut photos, mut videos), path| {
            if let Some(extension) = Path::new(&path).extension() {
                match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" => {
                        photos.push(path);
                    }
                    "mp4" | "mov" | "avi" | "mkv" | "flv" | "webm" => {
                        videos.push(path);
                    }
                    _ => {
                        warn!("Ignoring unknown file type: {}", path);
                    }
                }
            } else {
                warn!("Ignoring path with no extension: {}", path);
            }
            (photos, videos)
        },
    )
}
pub async fn process_thumbnails(
    image_relative_paths: Vec<String>,
    settings: Settings,
) -> Result<(), loco_rs::Error> {
    // Configuration constants
    const MAX_STATUS_RETRIES: u64 = 5;
    const BASE_RETRY_DELAY_MS: u64 = 1000;
    const POLL_INTERVAL_SEC: u64 = 5;
    const JOB_TIMEOUT_SEC: u64 = 300; // 5 minutes

    let client = ThumbnailClient::new(&settings.processing_api_url);
    
    // Partition media paths by type
    let (photo_paths, video_paths) = split_media_paths(image_relative_paths);
    let job_request = ThumbnailRequest {
        photos: photo_paths,
        videos: video_paths,
    };
    
    // Submit job with standard error handling
    let job_id = client
        .submit_job(&job_request)
        .await
        .map_err(|e| loco_rs::Error::Message(format!("Failed to submit thumbnail job: {}", e)))?;
    
    info!("Submitted thumbnail job: {}", job_id);
    
    // Poll for completion with timeout
    let start_time = std::time::Instant::now();
    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SEC)).await;
        
        // Check status with exponential backoff retry pattern
        let status = retry_with_backoff(MAX_STATUS_RETRIES, BASE_RETRY_DELAY_MS, || async {
            client
                .check_status(&job_id)
                .await
                .map_err(|e| loco_rs::Error::Message(format!("Status check failed: {}", e)))
        })
        .await?;
        
        info!(
            "Job {} progress: {}/{} photos, {}/{} videos",
            job_id,
            status.photos_done,
            status.photos_total,
            status.videos_done,
            status.videos_total
        );
        
        if status.done {
            info!("Job {} completed successfully", job_id);
            
            // Clean up the job with retry logic
            retry_with_backoff(3, BASE_RETRY_DELAY_MS, || async {
                client
                    .delete_job(&job_id)
                    .await
                    .map_err(|e| loco_rs::Error::Message(format!("Failed to delete job: {}", e)))
            })
            .await?;
            
            info!("Deleted completed job {}", job_id);
            return Ok(());
        }
        
        // Check for timeout
        if start_time.elapsed().as_secs() >= JOB_TIMEOUT_SEC {
            error!("Job {} timed out after {} seconds", job_id, JOB_TIMEOUT_SEC);
            return Err(loco_rs::Error::Message(
                format!("Generate thumbnail request timed out after {} seconds", JOB_TIMEOUT_SEC)
            ));
        }
    }
}

// Generic retry function with exponential backoff and jitter
async fn retry_with_backoff<F, Fut, T, E>(max_retries: u64, base_delay_ms: u64, operation: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    use rand::{thread_rng, Rng};
    
    let mut retries = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if retries >= max_retries {
                    return Err(e);
                }
                
                retries += 1;
                
                // Calculate exponential backoff with jitter
                let delay_ms = base_delay_ms * (1 << retries);
                let jitter = thread_rng().gen_range(0..=delay_ms / 2);
                let backoff_ms = delay_ms + jitter;
                
                info!("Operation failed, retry {}/{} after {}ms", retries, max_retries, backoff_ms);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }
}
