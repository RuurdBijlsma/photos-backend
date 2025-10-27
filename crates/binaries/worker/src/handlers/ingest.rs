use crate::context::WorkerContext;
use crate::handlers::JobResult;
use crate::handlers::db::store_media::store_media_item;
use crate::jobs::management::is_job_cancelled;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use common_photos::{
    Job, get_thumb_options, media_dir, nice_id, relative_path_abs, settings, thumbnails_dir,
};
use generate_thumbnails::generate_thumbnails;

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

    generate_thumbnails(&file_path, &thumbnail_out_dir, &thumb_config, media_info.metadata.orientation).await?;

    if !file_path.exists() {
        // File deleted while thumbs where generating
        return Ok(JobResult::Cancelled);
    }

    let mut tx = context.pool.begin().await?;

    let job_result = if is_job_cancelled(&mut tx, job.id).await? {
        JobResult::Cancelled
    } else {
        store_media_item(
            &mut tx,
            &relative_path_abs(&file_path)?,
            &media_info,
            &media_item_id,
            user_id,
        )
        .await?;

        JobResult::Done
    };
    tx.commit().await?;

    Ok(job_result)
}
