use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use common_services::database::album::pending_album_media_item::PendingAlbumMediaItem;
use common_services::database::jobs::Job;
use common_services::database::media_item::media_item::FromAnalyzerResult;
use common_services::database::media_item_store::MediaItemStore;
use common_services::get_settings::{media_dir, settings, thumbnails_dir};
use common_services::utils::{get_thumb_options, nice_id, relative_path_abs};
use generate_thumbnails::generate_thumbnails;
use sqlx::PgTransaction;

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
        MediaItemStore::create(
            &mut tx,
            &FromAnalyzerResult {
                result: media_info,
                media_item_id,
                user_id,
                relative_path: relative_path_abs(&file_path)?,
            }
            .into(),
            remote_user_id,
        )
        .await?;

        JobResult::Done
    };
    tx.commit().await?;

    Ok(job_result)
}

async fn get_or_create_remote_user(
    tx: &mut PgTransaction<'_>,
    local_user_id: i32,
    remote_identity: &str,
) -> std::result::Result<i32, sqlx::Error> {
    let remote_user_id = sqlx::query_scalar!(
        "SELECT id FROM remote_user WHERE identity = $1 AND user_id = $2",
        remote_identity,
        local_user_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(id) = remote_user_id {
        return Ok(id);
    }

    // Not found, so create it
    let new_id = sqlx::query_scalar!(
        "INSERT INTO remote_user (identity, user_id) VALUES ($1, $2) RETURNING id",
        remote_identity,
        local_user_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(new_id)
}
