use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use common_services::queue::{Job, enqueue_full_ingest};
use common_services::settings::media_dir;
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

    // 2. Determine file name and save path
    let album_name = query!(
        "SELECT name FROM album WHERE id = $1",
        payload.local_album_id
    )
    .fetch_one(&context.pool)
    .await?
    .name;

    let content_disposition = response
        .headers()
        .get("content-disposition")
        .and_then(|val| val.to_str().ok())
        .unwrap_or("")
        .to_string(); // This makes a copy and releases the borrow

    let filename = content_disposition.split("filename=").last().map_or_else(
        || payload.remote_media_item_id.clone(),
        |s| s.trim_matches('"').to_owned(),
    );

    let user_media_folder = query!("SELECT media_folder FROM app_user WHERE id = $1", user_id)
        .fetch_one(&context.pool)
        .await?
        .media_folder
        .ok_or_else(|| eyre!("User has no media folder configured"))?;

    // Sanitize album name to be a valid directory name
    let sanitized_album_name = sanitize_filename::sanitize(&album_name);
    let relative_dir = Path::new(&user_media_folder).join(&sanitized_album_name);
    let full_save_dir = media_dir().join(&relative_dir);
    fs::create_dir_all(&full_save_dir).await?;

    let full_save_path = full_save_dir.join(&filename);
    let mut dest_file = fs::File::create(&full_save_path).await?;

    // This move is now valid because `response` is no longer borrowed.
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        dest_file.write_all(&item?).await?;
    }

    // 3. Create a pending record
    let relative_path = to_posix_string(&relative_dir.join(&filename));

    query!(
        r#"
        INSERT INTO pending_album_media_items (relative_path, album_id, remote_user_identity)
        VALUES ($1, $2, $3)
        "#,
        relative_path,
        payload.local_album_id.clone(),
        //todo: make nicer remote_user_identity (no https://)
        format!("{}@{}", payload.remote_username, payload.remote_url),
    )
    .execute(&context.pool)
    .await?;

    // 4. Enqueue a standard ingest job
    enqueue_full_ingest(&context.pool, &relative_path, user_id).await?;

    Ok(JobResult::Done)
}
