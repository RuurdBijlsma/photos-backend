use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::cache::{
    get_thumbnail_cache, write_thumbnail_cache,
};
use crate::jobs::management::is_job_cancelled;
use color_eyre::{Result, eyre::eyre};
use common_services::database::jobs::Job;
use generate_thumbnails::{copy_dir_contents, generate_thumbnails};
use std::path::Path;
use tracing::debug;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let relative_path = job
        .relative_path
        .as_deref()
        .ok_or_else(|| eyre!("Ingest job has no associated relative_path"))?;
    let media_root = &context.settings.ingest.media_root;
    let file_path = media_root.join(relative_path);
    let row = sqlx::query!(
        "SELECT id, hash, orientation FROM media_item WHERE relative_path = $1",
        relative_path
    )
    .fetch_one(&context.pool)
    .await?;
    if !file_path.exists() {
        return Ok(JobResult::Cancelled);
    }
    process_thumbnails(context, &file_path, &row.hash, &row.id, row.orientation).await?;
    if !file_path.exists() || is_job_cancelled(&context.pool, job.id).await? {
        return Ok(JobResult::Cancelled);
    }
    Ok(JobResult::Done)
}

/// Handles thumbnail creation. Checks cache first, generates if missing.
async fn process_thumbnails(
    context: &WorkerContext,
    file_path: &Path,
    file_hash: &str,
    media_item_id: &str,
    orientation: i32,
) -> Result<()> {
    let thumbnail_root = &context.settings.ingest.thumbnail_root;
    let thumbnails_out_folder = thumbnail_root.join(media_item_id);

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
