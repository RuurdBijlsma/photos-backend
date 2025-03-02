use crate::common::image_utils::{is_image_file, is_video_file};
use crate::common::settings::Settings;
use crate::common::{
    api_client::ApiClient,
    job_polling::{poll_job, JobStatus},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::warn;

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
impl JobStatus for ThumbnailJob {
    fn is_done(&self) -> bool {
        self.done
    }
}

pub async fn process_thumbnails(
    image_relative_paths: Vec<String>,
    settings: &Settings,
) -> Result<(), loco_rs::Error> {
    let client = ApiClient::new(&settings.processing_api_url, "thumbnails");
    let (photos, videos) = split_media_paths(image_relative_paths);

    let job_id = client
        .submit_job(&ThumbnailRequest { photos, videos })
        .await
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;

    let _status: ThumbnailJob = poll_job(
        &client, &job_id, 5,   // delay_secs
        300, // timeout_secs
        5,   // max_retries
        1,   // retry_delay_secs
    )
    .await?;

    client
        .delete_job(&job_id)
        .await
        .map_err(|e| loco_rs::Error::Message(e.to_string()))?;

    Ok(())
}

fn split_media_paths(paths: Vec<String>) -> (Vec<String>, Vec<String>) {
    paths.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut photos, mut videos), path| {
            let path_obj = Path::new(&path);

            // First check if we can determine the file type
            if is_image_file(path_obj) {
                photos.push(path);
            } else if is_video_file(path_obj) {
                videos.push(path);
            } else {
                match path_obj.extension() {
                    Some(_) => warn!("Ignoring unknown file type: {}", path),
                    None => warn!("Ignoring path with no extension: {}", path),
                }
            }

            (photos, videos)
        },
    )
}
