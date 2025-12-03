use crate::context::WorkerContext;
use crate::handlers::common::cache::{
    get_ingest_cache, get_thumbnail_cache, hash_file, write_ingest_cache, write_thumbnail_cache,
};
use crate::handlers::common::remote_user::get_or_create_remote_user;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use app_state::constants;
use color_eyre::{eyre::eyre, Result};
use common_services::database::album::pending_album_media_item::PendingAlbumMediaItem;
use common_services::database::album_store::AlbumStore;
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::utils::nice_id;
use generate_thumbnails::{copy_dir_contents, generate_thumbnails};
use media_analyzer::AnalyzeResult;
use sqlx::PgPool;
use std::path::Path;
use tokio::fs;
use tracing::{debug};

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    // 1. Validate Job Data
    let relative_path = job
        .relative_path
        .as_deref()
        .ok_or_else(|| eyre!("Ingest job has no associated relative_path"))?;
    let user_id = job
        .user_id
        .ok_or_else(|| eyre!("Ingest job has no associated user_id"))?;

    let media_root = &context.settings.ingest.media_root;
    let file_path = media_root.join(relative_path);

    if !file_path.exists() {
        return Ok(JobResult::Cancelled);
    }

    // 2. Prepare Identifiers
    let file_hash = hash_file(&file_path)?;
    let media_item_id = nice_id(constants().database.media_item_id_length);

    // 3. Get Media Info (Cache or Compute)
    let media_info = get_media_info(context, &file_path, &file_hash).await?;

    // 4. Process Thumbnails (Cache or Generate)
    process_thumbnails(
        context,
        &file_path,
        &file_hash,
        &media_item_id,
        media_info.metadata.orientation,
    )
        .await?;

    // 5. Final Checks before DB Write
    if !file_path.exists() || is_job_cancelled(&context.pool, job.id).await? {
        return Ok(JobResult::Cancelled);
    }

    // 6. Save to Database
    let deleted_id = store_media_item(
        &context.pool,
        user_id,
        relative_path,
        media_info,
        &media_item_id,
    )
        .await?;

    // 7. Cleanup
    cleanup_old_thumbnails(&context.settings.ingest.thumbnail_root, deleted_id).await?;

    Ok(JobResult::Done)
}

/// Retrieves media analysis. Checks cache first, computes if missing.
async fn get_media_info(
    context: &WorkerContext,
    file_path: &Path,
    file_hash: &str,
) -> Result<AnalyzeResult> {
    if context.settings.ingest.enable_cache
        && let Some(cached) = get_ingest_cache(file_hash).await? {
            debug!("Using ingest cache for {:?}", file_path.file_name());
            return Ok(cached);
        }

    let media_info = {
        let mut analyzer = context.media_analyzer.lock().await;
        analyzer.analyze_media(file_path).await?
    };

    if context.settings.ingest.enable_cache {
        write_ingest_cache(file_hash, &media_info).await?;
    }

    Ok(media_info)
}

/// Handles thumbnail creation. Checks cache first, generates if missing.
async fn process_thumbnails(
    context: &WorkerContext,
    file_path: &Path,
    file_hash: &str,
    media_item_id: &str,
    orientation: Option<u64>,
) -> Result<()> {
    let thumbnail_root = &context.settings.ingest.thumbnail_root;
    let thumbnails_out_folder = thumbnail_root.join(media_item_id);

    // Try Cache
    if context.settings.ingest.enable_cache
        && let Some(cached_folder) = get_thumbnail_cache(file_hash).await? {
            debug!(
                "Using thumbnail cache for {:?}: {}",
                file_path.file_name(),
                cached_folder.display()
            );
            copy_dir_contents(&cached_folder, &thumbnails_out_folder).await?;
            return Ok(());
        }

    // Cache Miss: Generate
    generate_thumbnails(
        &context.settings.ingest,
        file_path,
        &thumbnails_out_folder,
        orientation,
    )
        .await?;

    // Write Cache
    if context.settings.ingest.enable_cache {
        write_thumbnail_cache(file_hash, &thumbnails_out_folder).await?;
    }

    Ok(())
}

/// Transactional database update:
/// - Checks pending album items
/// - Deletes old media item (if re-ingesting)
/// - Creates new media item
/// - Updates album links
async fn store_media_item(
    pool: &PgPool,
    user_id: i32,
    relative_path: &str,
    analyze_result: AnalyzeResult,
    new_id: &str,
) -> Result<Option<String>> {
    let mut tx = pool.begin().await?;

    // Check for pending album link
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

    // Resolve remote user if applicable
    let remote_user_id = if let Some(info) = &pending {
        Some(get_or_create_remote_user(&mut tx, user_id, &info.remote_user_identity).await?)
    } else {
        None
    };

    // Remove old entry if this is a re-ingest
    let deleted_id = MediaItemStore::delete_by_relative_path(&mut *tx, relative_path).await?;

    // Create new entry
    MediaItemStore::create(
        &mut tx,
        new_id,
        relative_path,
        user_id,
        remote_user_id,
        &analyze_result.into(),
    )
        .await?;

    // Link to album if pending existed
    if let Some(info) = &pending {
        AlbumStore::add_media_items(
            &mut *tx,
            &info.album_id,
            &[new_id.to_string()],
            user_id,
        )
            .await?;
    }

    tx.commit().await?;
    Ok(deleted_id)
}

/// Delete replaced thumbnails from the filesystem
async fn cleanup_old_thumbnails(thumbnail_root: &Path, old_id: Option<String>) -> Result<()> {
    if let Some(id) = old_id {
        let path = thumbnail_root.join(id);
        if path.exists() {
            fs::remove_dir_all(path).await?;
        }
    }
    Ok(())
}