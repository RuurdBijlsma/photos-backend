use crate::context::WorkerContext;
use crate::handlers::common::remote_user::get_or_create_remote_user;
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
use std::path::{Path, PathBuf};
use std::slice;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let payload_value = job
        .payload
        .as_ref()
        .ok_or_else(|| eyre!("ImportAlbumItem job is missing a payload"))?;

    let payload: ImportAlbumItemPayload = from_value(payload_value.clone())?;
    let user_id = job
        .user_id
        .ok_or_else(|| eyre!("ImportAlbumItem Job missing user_id"))?;

    let remote_url = {
        let mut url = payload.remote_url.clone();
        url.set_path(&format!(
            "/s2s/albums/files/{}",
            payload.remote_media_item_id
        ));
        url
    };

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

    let filename = response
        .headers()
        .get("content-disposition")
        .and_then(|val| val.to_str().ok())
        .and_then(|cd| cd.split("filename=").last())
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| payload.remote_media_item_id.clone());

    let user_media_folder = query!("SELECT media_folder FROM app_user WHERE id = $1", user_id)
        .fetch_one(&context.pool)
        .await?
        .media_folder
        .ok_or_else(|| eyre!("User has no media folder configured"))?;
    let remote_host = payload
        .remote_url
        .host_str()
        .ok_or_else(|| eyre!("Remote URL is missing a host"))?;
    let remote_identity = format!("{}@{}", payload.remote_username, remote_host);
    let sanitized_identity = sanitize_filename::sanitize(&remote_identity);
    let relative_dir = Path::new(&user_media_folder)
        .join("import")
        .join(&sanitized_identity);
    let relative_path = to_posix_string(&relative_dir.join(&filename));

    if let Some(existing_id) =
        MediaItemStore::find_id_by_relative_path(&context.pool, &relative_path).await?
    {
        let mut tx = context.pool.begin().await?;
        AlbumStore::add_media_items(
            &mut *tx,
            &payload.local_album_id,
            slice::from_ref(&existing_id),
            user_id,
        )
        .await?;
        let remote_user_id = get_or_create_remote_user(&mut tx, user_id, &remote_identity).await?;
        MediaItemStore::update_remote_user_id(&mut *tx, &existing_id, remote_user_id).await?;
        tx.commit().await?;
        return Ok(JobResult::Done);
    }

    let full_save_dir = media_dir().join(&relative_dir);
    fs::create_dir_all(&full_save_dir).await?;
    let full_save_path = full_save_dir.join(&filename);

    // --- temp file ---
    let temp = NamedTempFile::new()?;
    let temp_path: PathBuf = temp.path().to_path_buf();
    let mut temp_file = fs::File::create(&temp_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        temp_file.write_all(&chunk?).await?;
    }
    temp_file.flush().await?;

    // --- move temp → destination ---
    fs::rename(&temp_path, &full_save_path).await?;

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

    enqueue_full_ingest(&context.pool, &relative_path, user_id).await?;

    Ok(JobResult::Done)
}
