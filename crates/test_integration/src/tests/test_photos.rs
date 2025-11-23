use crate::runner::context::test_context::TestContext;
use crate::test_helpers::{login, media_dir_contents};
use app_state::MakeRelativePath;
use color_eyre::eyre::{Context, ContextCompat, Result};
use reqwest::StatusCode;
use tokio::fs;

pub async fn test_photo_download(context: &TestContext) -> Result<()> {
    // ARRANGE
    let token = login(context).await?;
    let (photos, videos) = media_dir_contents(context)?;
    let client = &context.http_client;
    let url = format!("{}/photos/download", context.settings.api.public_url);

    // --- TEST 1: Download a Photo (Happy Path) ---
    if let Some(photo_path) = photos.first() {
        let relative_path = photo_path.make_relative(&context.settings.ingest.media_root)?;

        let response = client
            .get(&url)
            .query(&[("path", &relative_path)])
            .bearer_auth(&token)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        // Check Headers
        let content_type = response
            .headers()
            .get("content-type")
            .expect("content type header")
            .to_str()?;
        assert!(content_type.starts_with("image/"), "Expected image mime type, got {content_type}");

        // Check Body Size matches File Size
        let file_size = fs::metadata(photo_path).await?.len();
        let bytes = response.bytes().await?;
        assert_eq!(bytes.len() as u64, file_size, "Downloaded byte count mismatch for photo");
    } else {
        println!("Skipping photo download test (no photos found in assets)");
    }

    // --- TEST 2: Download a Video (Happy Path) ---
    if let Some(video_path) = videos.first() {
        let relative_path = video_path.make_relative(&context.settings.ingest.media_root)?;

        let response = client
            .get(&url)
            .query(&[("path", &relative_path)])
            .bearer_auth(&token)
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let file_size = fs::metadata(video_path).await?.len();
        let bytes = response.bytes().await?;
        assert_eq!(bytes.len() as u64, file_size, "Downloaded byte count mismatch for video");
    } else {
        println!("Skipping video download test (no videos found in assets)");
    }

    // --- TEST 3: File Not Found ---
    let response = client
        .get(&url)
        .query(&[("path", "non_existent_file.jpg")])
        .bearer_auth(&token)
        .send()
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // --- TEST 4: Path Traversal Attack (Security) ---
    let media_root = &context.settings.ingest.media_root;
    let parent_dir = media_root.parent().context("Media root has no parent")?;
    let secret_file = parent_dir.join("secret_system_file.txt");
    fs::write(&secret_file, "super secret content").await?;

    let traversal_path = "../secret_system_file.txt";

    let response = client
        .get(&url)
        .query(&[("path", traversal_path)])
        .bearer_auth(&token)
        .send()
        .await?;

    // Cleanup
    let _ = fs::remove_file(secret_file).await;

    // The service checks `!file_canon.starts_with(&media_dir_canon)` -> returns InvalidPath (400)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Traversal attempt should return 400 Bad Request");

    // --- TEST 5: Unauthorized Access ---
    if let Some(photo_path) = photos.first() {
        let relative_path = photo_path.make_relative(&context.settings.ingest.media_root)?;

        let response = client
            .get(&url)
            .query(&[("path", &relative_path)])
            // No Bearer Auth
            .send()
            .await?;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    Ok(())
}