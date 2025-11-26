use crate::runner::context::test_context::TestContext;
use crate::test_helpers::{login, media_dir_contents};
use app_state::MakeRelativePath;
use color_eyre::eyre::bail;
use color_eyre::Result;
use common_services::api::onboarding::interfaces::{
    DiskResponse, MakeFolderBody, MediaSampleResponse, StartProcessingBody,
    UnsupportedFilesResponse,
};
use futures_util::StreamExt;
use reqwest::StatusCode;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tracing::info;

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
    let created_folder = "integration_test_folder";
    let response = client
        .post(format!("{api_url}/onboarding/make-folder"))
        .bearer_auth(&token)
        .json(&MakeFolderBody {
            base_folder: String::new(),
            new_name: created_folder.to_string(),
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
    assert!(folders.contains(&created_folder.to_string()));
    let folder = &context.settings.ingest.media_root.join(created_folder);
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

    // 2. Prepare expected counts
    let (photos, videos) = media_dir_contents(context)?;
    let expected_media_items = photos.len() + videos.len();

    // 3. Connect to WebSocket to listen for timeline events
    let ws_url = format!("{}/timeline/ws", api_url.replace("http", "ws"));
    let mut request = ws_url.into_client_request()?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        format!("access_token, {token}").parse()?,
    );

    let (mut socket, _) = connect_async(request).await?;
    info!("WebSocket connected for timeline updates");

    // 4. Start Processing
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

    // 5. Listen for WebSocket messages
    let timeout = Duration::from_secs(60);
    let start = Instant::now();
    let mut received_count = 0;

    info!(
        "Waiting for {} media items via WebSocket...",
        expected_media_items
    );

    while received_count < expected_media_items {
        if start.elapsed() > timeout {
            bail!(
                "Processing media files took longer than the timeout: {:?}. Received {}/{}",
                timeout,
                received_count,
                expected_media_items
            );
        }

        // Wait for next message with a short timeout to allow checking global timeout loop
        match tokio::time::timeout(Duration::from_secs(1), socket.next()).await {
            Ok(Some(Ok(msg))) => {
                if msg.is_text() {
                    received_count += 1;
                    info!(
                        "WebSocket event received: {}/{}",
                        received_count, expected_media_items
                    );
                }
            }
            Ok(Some(Err(e))) => bail!("WebSocket error: {}", e),
            Ok(None) => bail!("WebSocket closed unexpectedly"),
            Err(_) => {}
        }
    }
    info!("All media items are processed");

    // 6. Check if thumbnails are actually there.
    {
        struct MediaItem {
            id: String,
            relative_path: String,
        }
        let media_items = sqlx::query_as!(MediaItem, "SELECT id, relative_path FROM media_item")
            .fetch_all(&context.pool)
            .await?;
        for item in &media_items {
            let path = context.settings.ingest.media_root.join(&item.relative_path);
            let thumbs_exist = context.settings.ingest.thumbs_exist(&path, &item.id)?;
            assert!(thumbs_exist);
        }
    }

    // 7. Check if media item relative paths match actual files in media root.
    let db_paths: HashSet<String> = sqlx::query_scalar!("SELECT relative_path FROM media_item")
        .fetch_all(&context.pool)
        .await?
        .into_iter()
        .collect();
    let fs_paths: HashSet<_> = photos
        .into_iter()
        .chain(videos.into_iter())
        .map(|p| p.make_relative(&context.settings.ingest.media_root))
        .collect::<Result<_>>()?;
    assert_eq!(db_paths, fs_paths);
    Ok(())
}
