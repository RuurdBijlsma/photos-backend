use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::cache::{
    get_thumbnail_cache, write_thumbnail_cache,
};
use crate::jobs::management::is_job_cancelled;
use color_eyre::{Result, eyre::eyre};
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use generate_thumbnails::{copy_dir_contents, generate_thumbnails};
use serde_json::Value;
use sqlx::PgPool;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, warn};

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let relative_path = job
        .relative_path
        .as_deref()
        .ok_or_else(|| eyre!("Ingest job has no associated relative_path"))?;
    let media_root = &context.settings.ingest.media_root;
    let file_path = media_root.join(relative_path);
    let Some(row) = sqlx::query!(
        "SELECT id, hash, orientation, use_panorama_viewer FROM media_item WHERE relative_path = $1",
        relative_path
    )
    .fetch_optional(&context.pool)
    .await?
    else {
        warn!("IngestThumbnail was called but no media_item row exists for {relative_path}");
        return Ok(JobResult::Cancelled);
    };
    if !file_path.exists() {
        return Ok(JobResult::Cancelled);
    }
    process_thumbnails(
        context,
        &file_path,
        &row.hash,
        &row.id,
        row.use_panorama_viewer,
        row.orientation,
    )
    .await?;
    if !file_path.exists() || is_job_cancelled(&context.pool, job.id).await? {
        return Ok(JobResult::Cancelled);
    }
    sqlx::query!(
        r"
        UPDATE media_item
        SET has_thumbnails = TRUE,
        updated_at = now()
        WHERE id = $1;",
        &row.id
    )
    .execute(&context.pool)
    .await?;
    Ok(JobResult::Done)
}

async fn store_panorama_config(
    pool: &PgPool,
    pano_sub_folder: &Path,
    media_item_id: &str,
) -> Result<()> {
    let pano_config_path = pano_sub_folder.join("config.json");
    if !pano_config_path.exists() {
        return Err(eyre!("Pano config file does not exist"));
    }
    let file = File::open(&pano_config_path)?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;
    MediaItemStore::upsert_panorama_config(pool, media_item_id, &json).await?;
    Ok(())
}

/// Handles thumbnail creation. Checks cache first, generates if missing.
async fn process_thumbnails(
    context: &WorkerContext,
    file_path: &Path,
    file_hash: &str,
    media_item_id: &str,
    use_panorama_viewer: bool,
    orientation: i32,
) -> Result<()> {
    let thumbnails_out_folder = context.settings.ingest.thumbnails_root.join(media_item_id);
    let pano_out_folder = context.settings.ingest.pano_root.join(media_item_id);

    // Try Cache
    if context.settings.ingest.enable_cache
        && let Some(cached_folder) = get_thumbnail_cache(file_hash).await?
    {
        debug!(
            "Using thumbnail cache for {:?}: {}",
            file_path.file_name(),
            cached_folder.display()
        );
        copy_dir_contents(&cached_folder, &thumbnails_out_folder).await?;
    } else {
        // Cache Miss: Generate
        generate_thumbnails(
            &context.settings.ingest,
            file_path,
            &thumbnails_out_folder,
            &pano_out_folder,
            use_panorama_viewer,
            orientation,
        )
        .await?;
        // Write Cache
        if context.settings.ingest.enable_cache {
            write_thumbnail_cache(file_hash, &thumbnails_out_folder).await?;
        }
    }
    if use_panorama_viewer {
        store_panorama_config(
            &context.pool,
            &pano_out_folder,
            media_item_id,
        )
        .await
        .ok();
    }

    Ok(())
}
