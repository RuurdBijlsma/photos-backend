use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::job_payloads::{ImportAlbumItemPayload, ImportAlbumPayload};
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use common_photos::{InviteSummaryResponse, Job, JobType, enqueue_job, nice_id, settings};
use serde_json::json;
use sqlx::query;
use url::Url;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(payload_value) = &job.payload else {
        return Err(eyre!("ImportAlbum job is missing a payload"));
    };
    let payload: ImportAlbumPayload = serde_json::from_value(payload_value.clone())?;
    let user_id = job.user_id.ok_or_else(|| eyre!("Job missing user_id"))?;

    // 1. Contact remote server to get the list of media items
    let parts: Vec<&str> = payload.token.split('-').collect();
    let host_part = parts.last().ok_or_else(|| eyre!("Invalid token format"))?;
    let host = host_part.split('@').next_back().unwrap();
    let mut remote_url = Url::parse(&format!("http://{host}"))
        .or_else(|_| Url::parse(&format!("https://{host}")))?;
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

    let summary: InviteSummaryResponse = response
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
            token: payload.token.clone(),
            local_album_id: album_id.clone(),
            remote_owner_identity: payload.remote_owner_identity.clone(),
            remote_server_url: host.to_string(),
        };

        enqueue_job(
            &context.pool,
            JobType::ImportAlbumItem,
            None,
            Some(user_id),
            Some(json!(item_payload)),
        )
        .await?;
    }

    Ok(JobResult::Done)
}
