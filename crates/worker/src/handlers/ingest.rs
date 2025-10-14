use crate::context::WorkerContext;
use crate::handlers::db::store_media::store_media_item;
use crate::handlers::JobResult;
use crate::jobs::management::is_job_cancelled;
use color_eyre::Result;
use common_photos::{
    get_thumb_options, media_dir, nice_id, relative_path_abs, settings, thumbnails_dir, Job,
};
use ruurd_photos_thumbnail_generation::generate_thumbnails;

pub async fn handle(context: &WorkerContext, job: &Job) -> Result<JobResult> {
    let file_path = media_dir().join(&job.relative_path);
    let thumb_config = get_thumb_options();
    let thumb_base_dir = thumbnails_dir();
    let media_item_id = nice_id(settings().database.media_item_id_length);
    let thumbnail_out_dir = thumb_base_dir.join(&media_item_id);

    generate_thumbnails(&file_path, &thumbnail_out_dir, &thumb_config).await?;

    if !file_path.exists() {
        // File deleted while thumbs where generating
        return Ok(JobResult::Cancelled);
    }

    let smallest_thumb_path = thumbnail_out_dir.join(format!(
        "{}p.avif",
        thumb_config.heights.iter().min().unwrap()
    ));

    let media_info = {
        let mut analyzer = context.media_analyzer.lock().await;
        analyzer
            .analyze_media(&file_path, &smallest_thumb_path)
            .await?
    };

    let mut tx = context.pool.begin().await?;

    let job_result = if is_job_cancelled(&mut tx, job.id).await? {
        JobResult::Cancelled
    } else {
        store_media_item(
            &mut tx,
            &relative_path_abs(&file_path)?,
            &media_info,
            &media_item_id,
            job.user_id,
        )
            .await?;

        JobResult::Done
    };
    tx.commit().await?;

    Ok(job_result)
}