// crates/binaries/worker/src/handlers/import_album_item.rs

use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use common_services::database::album_store::AlbumStore;
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::get_settings::media_dir;
use common_services::job_queue::enqueue_full_ingest;
use common_services::utils::to_posix_string;
use common_types::ImportAlbumItemPayload;
use futures_util::StreamExt;
use serde_json::from_value;
use sqlx::query;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(payload_value) = &job.payload else {
        return Err(eyre!("ImportAlbumItem job is missing a payload"));
    };
    let payload: ImportAlbumItemPayload = from_value(payload_value.clone())?;
    let user_id = job
        .user_id
        .ok_or_else(|| eyre!("ImportAlbumItem Job missing user_id"))?;

    // 1. Download the file from the remote server
    let mut remote_url = payload.remote_url.clone();
    remote_url.set_path(&format!(
        "/s2s/albums/files/{}",
        payload.remote_media_item_id
    ));

    let client = reqwest::Client::new();
    let response = client
        .get(remote_url)
        .bearer_auth(&payload.token)
        .send()
        .await
        .wrap_err("Failed to download file from remote server")?;

    if !response.status().is_success() {
        return Err(eyre!(
            "Remote server returned an error during file download: {}",
            response.status()
        ));
    }

    // 2. Determine file name and construct the new save path
    let content_disposition = response
        .headers()
        .get("content-disposition")
        .and_then(|val| val.to_str().ok())
        .unwrap_or("")
        .to_string();

    let filename = content_disposition.split("filename=").last().map_or_else(
        || payload.remote_media_item_id.clone(),
        |s| s.trim_matches('"').to_owned(),
    );

    let user_media_folder = query!("SELECT media_folder FROM app_user WHERE id = $1", user_id)
        .fetch_one(&context.pool)
        .await?
        .media_folder
        .ok_or_else(|| eyre!("User has no media folder configured"))?;

    // Format the remote URL to be just the host (e.g., "photos.example.com")
    let remote_host = payload
        .remote_url
        .host_str()
        .ok_or_else(|| eyre!("Remote URL is missing a host"))?;

    let remote_identity = format!("{}@{}", payload.remote_username, remote_host);
    let sanitized_identity_folder = sanitize_filename::sanitize(&remote_identity);
    let relative_dir = Path::new(&user_media_folder)
        .join("import")
        .join(&sanitized_identity_folder);
    let relative_path = to_posix_string(&relative_dir.join(&filename));

    if let Some(existing_id) =
        MediaItemStore::find_id_by_relative_path(&context.pool, &relative_path).await?
    {
        // File to import already exists, assume it's the same file and put it in the album.
        AlbumStore::add_media_items(
            &context.pool,
            &payload.local_album_id,
            &[existing_id],
            user_id,
        )
        .await?;
    } else {
        let full_save_dir = media_dir().join(&relative_dir);
        fs::create_dir_all(&full_save_dir).await?;

        let full_save_path = full_save_dir.join(&filename);
        let mut dest_file = fs::File::create(&full_save_path).await?;

        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            dest_file.write_all(&item?).await?;
        }

        // 3. Create a pending record
        query!(
            r#"
        INSERT INTO pending_album_media_items (relative_path, album_id, remote_user_identity)
        VALUES ($1, $2, $3)
        "#,
            relative_path,
            payload.local_album_id,
            remote_identity,
        )
        .execute(&context.pool)
        .await?;

        // 4. Enqueue a standard ingest job for the newly saved file
        enqueue_full_ingest(&context.pool, &relative_path, user_id).await?;
    }

    Ok(JobResult::Done)
}
