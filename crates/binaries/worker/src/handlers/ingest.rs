use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use common_services::database::album::pending_album_media_item::PendingAlbumMediaItem;
use common_services::database::album_store::AlbumStore;
use common_services::database::jobs::Job;
use common_services::database::media_item::media_item::FromAnalyzerResult;
use common_services::database::media_item_store::MediaItemStore;
use common_services::get_settings::{media_dir, settings, thumbnails_dir};
use common_services::utils::{get_thumb_options, nice_id, relative_path_abs};
use generate_thumbnails::generate_thumbnails;
use sqlx::PgTransaction;
use tokio::fs;
use crate::handlers::common::remote_user::get_or_create_remote_user;

/// Handles the ingestion of a media file, including thumbnail generation and database storage.
///
/// # Errors
///
/// This function will return an error if the job is missing required data,
/// thumbnail generation fails, media analysis fails, or a database operation fails.
pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(relative_path) = &job.relative_path else {
        return Err(eyre!("Ingest job has no associated relative_path"));
    };
    let Some(user_id) = job.user_id else {
        return Err(eyre!("Ingest job has no associated user_id"));
    };

    let file_path = media_dir().join(relative_path);
    let media_info = {
        let mut analyzer = context.media_analyzer.lock().await;
        analyzer.analyze_media(&file_path).await?
    };

    let thumb_config = get_thumb_options();
    let thumb_base_dir = thumbnails_dir();
    let media_item_id = nice_id(settings().database.media_item_id_length);
    let thumbnail_out_dir = thumb_base_dir.join(&media_item_id);

    generate_thumbnails(
        &file_path,
        &thumbnail_out_dir,
        &thumb_config,
        media_info.metadata.orientation,
    )
    .await?;

    if !file_path.exists() {
        // File deleted while thumbs where generating
        return Ok(JobResult::Cancelled);
    }

    let mut tx = context.pool.begin().await?;
    let mut old_media_item_id: Option<String> = None;

    let job_result = if is_job_cancelled(&mut tx, job.id).await? {
        JobResult::Cancelled
    } else {
        // todo: modularize this pending album media item logic more.
        let pending_info: Option<PendingAlbumMediaItem> = sqlx::query_as!(
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
        let remote_user_id = if let Some(info) = &pending_info {
            Some(get_or_create_remote_user(&mut tx, user_id, &info.remote_user_identity).await?)
        } else {
            None
        };
        old_media_item_id =
            MediaItemStore::delete_by_relative_path(&mut *tx, relative_path).await?;
        MediaItemStore::create(
            &mut tx,
            &FromAnalyzerResult {
                result: media_info,
                media_item_id: media_item_id.clone(),
                user_id,
                relative_path: relative_path_abs(&file_path)?,
            }
            .into(),
            remote_user_id,
        )
        .await?;
        if let Some(info) = &pending_info {
            AlbumStore::add_media_items(&mut *tx, &info.album_id, &[media_item_id], user_id)
                .await?;
        }

        JobResult::Done
    };
    tx.commit().await?;

    // Clean up replaced media item's thumbs.
    if let Some(id_to_delete) = old_media_item_id {
        let thumb_path_to_delete = thumbnails_dir().join(id_to_delete);
        if thumb_path_to_delete.exists() {
            fs::remove_dir_all(thumb_path_to_delete).await?;
        }
    }

    Ok(job_result)
}