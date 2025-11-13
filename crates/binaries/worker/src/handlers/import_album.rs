use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use serde_json::from_value;
use common_services::queue::{enqueue_job, Job, JobType};
use sqlx::query;
use common_services::settings::settings;
use common_services::utils::nice_id;
use common_types::{ImportAlbumItemPayload, ImportAlbumPayload};
use common_types::album::AlbumSummary;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(payload_value) = &job.payload else {
        return Err(eyre!("ImportAlbum job is missing a payload"));
    };
    let payload: ImportAlbumPayload = from_value(payload_value.clone())?;
    let user_id = job
        .user_id
        .ok_or_else(|| eyre!("ImportAlbum Job missing user_id"))?;
    let mut remote_url = payload.remote_url.clone();
    remote_url.set_path("/s2s/albums/invite-summary");

    let client = reqwest::Client::new();
    let response = client
        .get(remote_url.clone())
        .bearer_auth(&payload.token)
        .send()
        .await
        .wrap_err(format!(
            "Failed to contact remote server {remote_url} for invite summary."
        ))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(eyre!("Remote server returned an error: {}", error_text));
    }

    let summary: AlbumSummary = response
        .json()
        .await
        .wrap_err("Failed to parse summary from remote server")?;

    // 2. Create the new album locally
    let album_id = nice_id(settings().database.album_id_length);
    query!(
        r#"
        INSERT INTO album (id, owner_id, name, description)
        VALUES ($1, $2, $3, $4)
        "#,
        album_id,
        user_id,
        payload.album_name,
        payload.album_description
    )
    .execute(&context.pool)
    .await?;

    // 3. For each media item, enqueue a download & import job
    for remote_id in summary.media_item_ids {
        let item_payload = ImportAlbumItemPayload {
            remote_media_item_id: remote_id,
            local_album_id: album_id.clone(),
            remote_username: payload.remote_username.clone(),
            remote_url: payload.remote_url.clone(),
            token: payload.token.clone(),
        };

        enqueue_job(&context.pool, JobType::ImportAlbumItem)
            .user_id(user_id)
            .payload(&item_payload)
            .call()
            .await?;
    }

    Ok(JobResult::Done)
}
