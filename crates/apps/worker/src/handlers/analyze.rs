use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::common::cache::{get_analysis_cache, hash_file, write_analysis_cache};
use crate::jobs::management::is_job_cancelled;
use color_eyre::eyre::{Result, eyre};
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::database::visual_analysis_store::VisualAnalysisStore;
use common_types::ml_analysis::PyVisualAnalysis;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Handles the analysis of a given job.
///
/// # Errors
///
/// This function will return an error if the media analysis fails,
/// or if there are issues with database operations.
pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let Some(relative_path) = &job.relative_path else {
        return Err(eyre!("Ingest job has no associated relative_path"));
    };

    let media_root = &context.settings.ingest.media_root;
    let file_path = media_root.join(relative_path);

    // 1. Basic validation
    if !file_path.exists() {
        return Ok(JobResult::Cancelled);
    }

    if !context
        .settings
        .ingest
        .file_is_ingested(&context.pool, &file_path)
        .await?
    {
        info!(
            "File {} is not ingested yet, rescheduling analysis.",
            &relative_path
        );
        return Ok(JobResult::DependencyReschedule);
    }

    // 2. Resolve Media Item ID
    let media_item_id = MediaItemStore::find_id_by_relative_path(&context.pool, relative_path)
        .await?
        .ok_or_else(|| eyre!("Could not find media item by relative_path."))?;

    // 3. Get Analysis Data (Computed or Cached)
    let analyses = get_analysis_data(context, &file_path, &media_item_id).await?;

    // 4. Save results
    save_results(context, job.id, &media_item_id, &analyses, &file_path).await
}

/// Orchestrates retrieving analysis data.
/// Checks the cache first; if missing, computes it and writes to cache.
async fn get_analysis_data(
    context: &WorkerContext,
    file_path: &Path,
    media_item_id: &str,
) -> Result<Vec<PyVisualAnalysis>> {
    let file_hash = hash_file(file_path)?;

    // Try Cache
    if context.settings.ingest.enable_cache
        && let Some(cached_analysis) = get_analysis_cache(&file_hash).await?
    {
        debug!("Using analysis cache for {}", media_item_id);
        return Ok(cached_analysis);
    }

    // Cache Miss: Compute
    let analyses = compute_analysis(context, file_path, media_item_id).await?;

    // Write Cache
    if context.settings.ingest.enable_cache {
        write_analysis_cache(&file_hash, &analyses).await?;
    }

    Ok(analyses)
}

/// Performs the actual ML analysis by spawning blocking tasks.
async fn compute_analysis(
    context: &WorkerContext,
    file_path: &Path,
    media_item_id: &str,
) -> Result<Vec<PyVisualAnalysis>> {
    let images_to_analyze = get_images_to_analyze(context, file_path, media_item_id)?;
    let mut analyses = Vec::new();

    for (percentage, image_path) in images_to_analyze {
        let analyzer = context.visual_analyzer.clone();
        let analyzer_settings = context.settings.ingest.analyzer.clone();

        // This spawn blocking -> block_on is needed because analyze image does work on the
        // main thread and this disturbs the integration test.
        let analysis_result = tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                analyzer
                    .analyze_image(&analyzer_settings, &image_path, percentage)
                    .await
            })
        })
        .await??;

        analyses.push(analysis_result);
    }

    Ok(analyses)
}

/// Determines which thumbnail files should be sent to the ML analyzer.
fn get_images_to_analyze(
    context: &WorkerContext,
    file_path: &Path,
    media_item_id: &str,
) -> Result<Vec<(i32, PathBuf)>> {
    let thumbnail_root = &context.settings.ingest.thumbnail_root;
    let thumb_dir = thumbnail_root.join(media_item_id);

    if context.settings.ingest.is_photo_file(file_path) {
        let max_thumb = context
            .settings
            .ingest
            .thumbnails
            .heights
            .iter()
            .max()
            .ok_or_else(|| eyre!("Cannot find max thumbnail size"))?;
        Ok(vec![(0, thumb_dir.join(format!("{max_thumb}p.avif")))])
    } else {
        let items = context
            .settings
            .ingest
            .thumbnails
            .video_options
            .percentages
            .iter()
            .map(|p| {
                (
                    i32::try_from(*p).expect("Percentage should fit in i32"),
                    thumb_dir.join(format!("{p}_percent.avif")),
                )
            })
            .collect();
        Ok(items)
    }
}

/// Saves the analysis results to the database in a transaction.
/// Also checks for cancellation one last time before committing.
async fn save_results(
    context: &WorkerContext,
    job_id: i64,
    media_item_id: &str,
    analyses: &[PyVisualAnalysis],
    file_path: &Path,
) -> Result<JobResult> {
    let mut tx = context.pool.begin().await?;

    // Check cancellation or file existence one last time inside the transaction
    if is_job_cancelled(&mut *tx, job_id).await? || !file_path.exists() {
        return Ok(JobResult::Cancelled);
    }

    for analysis in analyses {
        VisualAnalysisStore::create(&mut tx, media_item_id, &analysis.to_owned().into()).await?;
    }

    tx.commit().await?;
    Ok(JobResult::Done)
}
