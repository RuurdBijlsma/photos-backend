use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use color_eyre::eyre::{Result, eyre};
use common_services::database::jobs::Job;
use common_services::database::media_item_store::MediaItemStore;
use common_services::database::visual_analysis_store::VisualAnalysisStore;
use common_services::get_settings::{media_dir, settings, thumbnails_dir};
use common_services::utils::{file_is_ingested, is_photo_file};
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
    let file_path = media_dir().join(relative_path);

    if !file_is_ingested(&file_path, &context.pool).await? {
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
    let thumb_dir = thumbnails_dir().join(&media_item_id);

    let images_to_analyze = if is_photo_file(&file_path) {
        let max_thumb = settings()
            .thumbnail_generation
            .heights
            .iter()
            .max()
            .ok_or_else(|| eyre!("Cannot find max thumbnail size"))?;
        vec![(0, thumb_dir.join(format!("{max_thumb}p.avif")))]
    } else {
        settings()
            .thumbnail_generation
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
        analyses.push(
            context
                .visual_analyzer
                .analyze_image(&image_path, percentage)
                .await?,
        );
    }

    let job_result = if is_job_cancelled(&mut *tx, job.id).await? {
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
