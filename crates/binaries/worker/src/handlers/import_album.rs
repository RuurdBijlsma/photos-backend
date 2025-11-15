use crate::context::WorkerContext;
use crate::handlers::JobResult;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use common_services::database::album::album::{AlbumRole, AlbumSummary};
use common_services::database::album_store::AlbumStore;
use common_services::database::jobs::{Job, JobType};
use common_services::get_settings::settings;
use common_services::job_queue::enqueue_job;
use common_services::s2s_client::client::S2sClient;
use common_services::utils::nice_id;
use common_types::{ImportAlbumItemPayload, ImportAlbumPayload};
use serde_json::from_value;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(payload_value) = &job.payload else {
        return Err(eyre!("ImportAlbum job is missing a payload"));
    };
    let payload: ImportAlbumPayload = from_value(payload_value.clone())?;
    let user_id = job
        .user_id
        .ok_or_else(|| eyre!("ImportAlbum Job missing user_id"))?;

    // 1. Contact the remote server for the album summary
    let http_client = reqwest::Client::new();
    let s2s_client = S2sClient::new(http_client);

    let summary: AlbumSummary = s2s_client
        .get_album_invite_summary(&payload.token)
        .await
        .wrap_err("Failed to get album invite summary from remote server")?;

    // 2. Create the new album locally
    let album_id = nice_id(settings().database.album_id_length);
    let mut tx = context.pool.begin().await?;
    AlbumStore::create(
        &mut *tx,
        &album_id,
        user_id,
        &payload.album_name,
        payload.album_description,
        false,
    )
        .await?;
    AlbumStore::upsert_collaborator(&mut *tx, &album_id, user_id, AlbumRole::Owner).await?;
    tx.commit().await?;

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