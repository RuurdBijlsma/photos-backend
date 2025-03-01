use crate::common::settings::Settings;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};

// -- Data structures --
#[derive(Debug, Serialize, Deserialize)]
struct ThumbnailRequest {
    photos: Vec<String>,
    videos: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailJob {
    job_id: String,
    photos_done: i32,
    photos_total: i32,
    videos_done: i32,
    videos_total: i32,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct HTTPValidationError {
    detail: Vec<ValidationError>,
}

#[derive(Debug, Deserialize)]
struct ValidationError {
    loc: Vec<String>,
    msg: String,
    #[serde(rename = "type")]
    error_type: String,
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
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error = response.json::<HTTPValidationError>().await?;
                Err(format!("Validation error: {:?}", error).into())
            }
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {}: {}", status, text).into())
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
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error = response.json::<HTTPValidationError>().await?;
                Err(format!("Validation error: {:?}", error).into())
            }
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {}: {}", status, text).into())
            }
        }
    }

    pub async fn delete_job(&self, job_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/thumbnails/{}", self.base_url, job_id);
        let response = self.client.delete(&url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error = response.json::<HTTPValidationError>().await?;
                Err(format!("Validation error: {:?}", error).into())
            }
            status => {
                let text = response.text().await?;
                Err(format!("Unexpected status {}: {}", status, text).into())
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
    let client = ThumbnailClient::new(&settings.processing_api_url);
    let (photo_paths, video_paths) = split_media_paths(image_relative_paths);

    let job_request = ThumbnailRequest {
        photos: photo_paths.iter().map(|s| s.to_string()).collect(),
        videos: video_paths.iter().map(|s| s.to_string()).collect(),
    };

    // Submit job
    let job_id = client
        .submit_job(&job_request)
        .await
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
    info!("Submitted thumbnail job: {}", job_id);

    // Poll every {delay} seconds
    let delay = 5;
    let timeout = 300; // 5 minutes
    let mut attempts = 0;

    loop {
        tokio::time::sleep(Duration::from_secs(delay)).await;

        let status = client
            .check_status(&job_id)
            .await
            .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
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

            // Clean up the job
            client
                .delete_job(&job_id)
                .await
                .map_err(|e| loco_rs::Error::Message(e.to_string()))?;
            info!("Deleted completed job {}", job_id);

            break Ok(());
        }

        attempts += 1;
        if attempts * delay >= timeout {
            error!("Job {} timed out after {} attempts", job_id, attempts);
            break Err(loco_rs::Error::Message(
                "Generate thumbnail request timed out".to_string(),
            ));
        }
    }
}
