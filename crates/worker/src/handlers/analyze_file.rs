use color_eyre::eyre::eyre;
use common_photos::{
    alert, file_is_ingested, is_photo_file, media_dir, settings, thumbnails_dir, Job,
};
use ml_analysis::VisualAnalyzer;
use sqlx::PgPool;
use std::fs::File;
use std::path::Path;
use tracing::warn;

pub async fn analyze_file(
    pool: &PgPool,
    job: &Job,
    visual_analyzer: &VisualAnalyzer,
) -> color_eyre::Result<()> {
    let mut tx = pool.begin().await?;
    let file = media_dir().join(&job.relative_path);
    if !file_is_ingested(&file, &mut *tx).await? {
        alert!("Analysis job picked up while file is not properly ingested");
        return Err(eyre!("File is not properly ingested"));
    }
    let media_item_id = sqlx::query_scalar!(
        r"
        SELECT id
        FROM media_item
        WHERE relative_path=$1
    ",
        &job.relative_path
    )
    .fetch_one(&mut *tx)
    .await?;
    let thumb_dir = thumbnails_dir().join(media_item_id);
    let to_analyze = if is_photo_file(&file) {
        let Some(biggest_thumb) = settings().thumbnail_generation.heights.iter().max() else {
            Err(eyre!("Can't retrieve max size of thumbnail."))?
        };
        vec![thumb_dir.join(format!("{biggest_thumb}p.avif"))]
    } else {
        settings()
            .thumbnail_generation
            .video_options
            .percentages
            .iter()
            .map(|p| thumb_dir.join(format!("{p}_percent.avif")))
            .collect::<Vec<_>>()
    };
    let mut analyses = vec![];
    for image in to_analyze {
        let res = visual_analyzer.analyze_image(&image).await?;
        analyses.push(res);
    }

    let file_filename = file.file_name().unwrap().to_string_lossy().to_string();
    let json_filename = format!("out-{file_filename}.json");
    let json_file = File::create(Path::new(&json_filename))?;
    serde_json::to_writer_pretty(json_file, &analyses)?;

    // put in db

    Ok(())
}
