use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::cache::{get_ingest_cache, hash_file, write_ingest_cache};
use crate::handlers::common::remote_user::get_or_create_remote_user;
use crate::jobs::management::is_job_cancelled;
use app_state::constants;
use color_eyre::eyre::Context;
use color_eyre::{Result, eyre::eyre};
use common_services::database::album::pending_album_media_item::PendingAlbumMediaItem;
use common_services::database::album_store::AlbumStore;
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::utils::nice_id;
use media_analyzer::MediaMetadata;
use sqlx::PgPool;
use std::path::Path;
use std::time::Instant;
use tracing::debug;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let start = Instant::now();
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
    println!("init_metadata_handler {:?}", start.elapsed());
    let now = Instant::now();
    let file_hash = hash_file(&file_path)?;
    let media_item_id = nice_id(constants().database.media_item_id_length);
    println!("hash+id {:?}", now.elapsed());
    let now = Instant::now();
    let media_info = get_media_info(context, &file_path, &file_hash).await?;
    println!("get_media_info {:?}", now.elapsed());
    let now = Instant::now();
    if !file_path.exists() || is_job_cancelled(&context.pool, job.id).await? {
        return Ok(JobResult::Cancelled);
    }
    store_media_item(
        &context.pool,
        user_id,
        relative_path,
        media_info,
        &media_item_id,
    )
    .await?;
    println!("store_media_item {:?}", now.elapsed());
    println!("Handle ingest metadata total: {:?}", start.elapsed());
    Ok(JobResult::Done)
}

/// Retrieves media analysis. Checks cache first, computes if missing.
async fn get_media_info(
    context: &WorkerContext,
    file_path: &Path,
    file_hash: &str,
) -> Result<MediaMetadata> {
    if context.settings.ingest.enable_cache
        && let Some(cached) = get_ingest_cache(file_hash).await?
    {
        println!("\tUse Cache");
        debug!("Using ingest cache for {:?}", file_path.file_name());
        return Ok(cached);
    }
    let now = Instant::now();
    let media_info = context
        .media_analyzer
        .analyze_media(file_path)
        .await
        .wrap_err(file_path.to_string_lossy().to_string())?;
    println!("\tAnalyze media: {:?}", now.elapsed());
    if context.settings.ingest.enable_cache {
        write_ingest_cache(file_hash, media_info.clone()).await?;
    }
    Ok(media_info)
}

async fn store_media_item(
    pool: &PgPool,
    user_id: i32,
    relative_path: &str,
    analyze_result: MediaMetadata,
    new_id: &str,
) -> Result<()> {
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
    MediaItemStore::delete_by_relative_path(&mut *tx, relative_path).await?;
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
    Ok(())
}
