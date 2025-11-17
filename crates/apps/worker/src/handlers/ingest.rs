use crate::context::WorkerContext;
use crate::handlers::common::remote_user::get_or_create_remote_user;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use std::path::Path;

use color_eyre::{eyre::eyre, Result};
use common_services::database::album::pending_album_media_item::PendingAlbumMediaItem;
use common_services::database::album_store::AlbumStore;
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::utils::nice_id;
use media_analyzer::AnalyzeResult;
use sqlx::PgPool;

use app_state::constants;
use generate_thumbnails::generate_thumbnails;
use tokio::fs;

/// Process pending album entry, delete old media, create new media entry
async fn store_media_item(
    pool: &PgPool,
    user_id: i32,
    relative_path: &str,
    analyze_result: AnalyzeResult,
    new_id: &str,
) -> Result<Option<String>> {
    let mut tx = pool.begin().await?;

    let pending = sqlx::query_as!(
        PendingAlbumMediaItem,
        r#"
        DELETE FROM pending_album_media_items
        WHERE relative_path = $1
        RETURNING album_id, remote_user_identity, relative_path
        "#,
        relative_path
    )
    .fetch_optional(&mut *tx)
    .await?;

    let remote_user_id = if let Some(info) = &pending {
        Some(get_or_create_remote_user(&mut tx, user_id, &info.remote_user_identity).await?)
    } else {
        None
    };

    let deleted_id = MediaItemStore::delete_by_relative_path(&mut *tx, relative_path).await?;

    MediaItemStore::create(
        &mut tx,
        new_id,
        relative_path,
        user_id,
        remote_user_id,
        &analyze_result.into(),
    )
    .await?;

    if let Some(info) = &pending {
        AlbumStore::add_media_items(&mut *tx, &info.album_id, &[new_id.to_string()], user_id)
            .await?;
    }

    tx.commit().await?;
    Ok(deleted_id)
}

/// Delete replaced thumbnails
async fn cleanup_old_thumbnails(thumbnail_root: &Path, old_id: Option<String>) -> Result<()> {
    if let Some(id) = old_id {
        let path = thumbnail_root.join(id);
        if path.exists() {
            fs::remove_dir_all(path).await?;
        }
    }
    Ok(())
}

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let relative_path = job
        .relative_path
        .as_deref()
        .ok_or_else(|| eyre!("Ingest job has no associated relative_path"))?;
    let user_id = job
        .user_id
        .ok_or_else(|| eyre!("Ingest job has no associated user_id"))?;
    let media_root = &context.settings.ingestion.media_folder;
    let thumbnail_root = &context.settings.ingestion.thumbnail_folder;
    let file_path = media_root.join(relative_path);
    if !file_path.exists() {
        return Ok(JobResult::Cancelled);
    }

    let media_info = {
        let mut analyzer = context.media_analyzer.lock().await;
        analyzer.analyze_media(&file_path).await?
    };
    let media_item_id = nice_id(constants().database.media_item_id_length);

    generate_thumbnails(
        &context.settings.ingestion,
        &file_path,
        &thumbnail_root.join(&media_item_id),
        media_info.metadata.orientation,
    )
    .await?;

    // Don't insert to DB if file doesn't exist, or job is cancelled.
    if !file_path.exists() || is_job_cancelled(&context.pool, job.id).await? {
        return Ok(JobResult::Cancelled);
    }
    let deleted_id = store_media_item(
        &context.pool,
        user_id,
        relative_path,
        media_info,
        &media_item_id,
    )
    .await?;
    cleanup_old_thumbnails(thumbnail_root, deleted_id).await?;

    Ok(JobResult::Done)
}
