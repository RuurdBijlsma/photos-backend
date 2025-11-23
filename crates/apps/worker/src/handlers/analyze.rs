use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use color_eyre::eyre::{Result, eyre};
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::database::visual_analysis_store::VisualAnalysisStore;
use tracing::info;

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
    let thumbnail_root = &context.settings.ingest.thumbnail_root;
    let file_path = media_root.join(relative_path);
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

    let mut tx = context.pool.begin().await?;
    let Some(media_item_id) =
        MediaItemStore::find_id_by_relative_path(&mut *tx, relative_path).await?
    else {
        return Err(eyre!("Could not find media item by relative_path."));
    };
    let thumb_dir = thumbnail_root.join(&media_item_id);

    let images_to_analyze = if context.settings.ingest.is_photo_file(&file_path) {
        let max_thumb = context
            .settings
            .ingest
            .thumbnails
            .heights
            .iter()
            .max()
            .ok_or_else(|| eyre!("Cannot find max thumbnail size"))?;
        vec![(0, thumb_dir.join(format!("{max_thumb}p.avif")))]
    } else {
        context
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
            .collect()
    };

    let mut analyses = Vec::new();
    for (percentage, image_path) in images_to_analyze {
        let analyzer = context.visual_analyzer.clone();
        let analyzer_settings = context.settings.ingest.analyzer.clone();

        // This spawn blocking -> block_on is needed because analyze image does heavy work on the
        // main thread and this disturbs the integration test.
        let analysis_result = tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                analyzer
                    .analyze_image(&analyzer_settings, &image_path, percentage)
                    .await
            })
        })
        .await??; // Double ? handles JoinError (panic) and the inner Result

        analyses.push(analysis_result);
    }

    let job_result = if is_job_cancelled(&mut *tx, job.id).await? || !file_path.exists() {
        JobResult::Cancelled
    } else {
        for analysis in &analyses {
            VisualAnalysisStore::create(&mut tx, &media_item_id, &analysis.to_owned().into())
                .await?;
        }
        JobResult::Done
    };

    tx.commit().await?;
    Ok(job_result)
}
