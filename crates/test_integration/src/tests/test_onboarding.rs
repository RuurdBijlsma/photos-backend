use crate::runner::context::test_context::TestContext;
use crate::test_helpers::{login, media_dir_contents};
use color_eyre::eyre::bail;
use color_eyre::Result;
use common_services::api::onboarding::interfaces::{
    DiskResponse, MakeFolderBody, MediaSampleResponse, StartProcessingBody,
    UnsupportedFilesResponse,
};
use reqwest::StatusCode;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::time::sleep;
use tracing::info;

const USER_FOLDER: &str = "integration_test_album";

pub async fn test_onboarding(context: &TestContext) -> Result<()> {
    // 1. Login
    let token = login(context).await?;
    let api_url = &context.settings.api.public_url;
    let client = &context.http_client;

    // 2. Get Disk Info
    let response = client
        .get(format!("{api_url}/onboarding/disk-info"))
        .bearer_auth(&token)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let disk_info: DiskResponse = response.json().await?;
    assert!(disk_info.media_folder.read_access);
    assert!(disk_info.thumbnails_folder.read_access);

    // 3. List Folders (root)
    let response = client
        .get(format!("{api_url}/onboarding/folders"))
        .query(&[("folder", "")])
        .bearer_auth(&token)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let _folders: Vec<String> = response.json().await?;

    // 4. Create a new folder
    let response = client
        .post(format!("{api_url}/onboarding/make-folder"))
        .bearer_auth(&token)
        .json(&MakeFolderBody {
            base_folder: String::new(),
            new_name: USER_FOLDER.to_string(),
        })
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. Verify the folder exists
    let response = client
        .get(format!("{api_url}/onboarding/folders"))
        .query(&[("folder", "")])
        .bearer_auth(&token)
        .send()
        .await?;

    let folders: Vec<String> = response.json().await?;
    assert!(folders.contains(&USER_FOLDER.to_string()));
    let folder = &context.settings.ingest.media_root.join(USER_FOLDER);
    assert!(folder.exists());
    fs::remove_dir(folder).await?;

    // 6. Check Media Sample
    let response = client
        .get(format!("{api_url}/onboarding/media-sample"))
        .query(&[("folder", "")])
        .bearer_auth(&token)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let sample: MediaSampleResponse = response.json().await?;
    let (photos, videos) = media_dir_contents(context)?;
    assert_eq!(sample.photo_count, photos.len());
    assert_eq!(sample.video_count, videos.len());

    // 7. Check Unsupported Files
    let response = client
        .get(format!("{api_url}/onboarding/unsupported-files"))
        .query(&[("folder", "")])
        .bearer_auth(&token)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let unsupported: UnsupportedFilesResponse = response.json().await?;
    assert_eq!(unsupported.unsupported_count, 0);

    Ok(())
}

pub async fn test_start_processing(context: &TestContext) -> Result<()> {
    // 1. Login
    let token = login(context).await?;
    let api_url = &context.settings.api.public_url;
    let client = &context.http_client;

    // 2. Start Processing
    // This sets the user's media folder and enqueues a scan job.
    let response = client
        .post(format!("{api_url}/onboarding/start-processing"))
        .bearer_auth(&token)
        .json(&StartProcessingBody {
            user_folder: String::new(),
        })
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let timeout = Duration::from_secs(120);
    let start = Instant::now();
    let (photos, videos) = media_dir_contents(context)?;
    let expected_media_items = photos.len() + videos.len();
    loop {
        let media_items = sqlx::query_scalar!("SELECT id FROM media_item")
            .fetch_all(&context.pool)
            .await?;
        info!(
            "{}/{} files processed.",
            media_items.len(),
            expected_media_items
        );
        if media_items.len() >= expected_media_items {
            break;
        }
        if start.elapsed() > timeout {
            bail!(
                "Processing media files took longer than the timeout: {:?}",
                timeout
            );
        }
        sleep(Duration::from_secs(10)).await;
    }
    info!("All media items are processed");
    // Todo: check if thumbnails and db items are there.

    Ok(())
}
