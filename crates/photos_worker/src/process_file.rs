use media_analyzer::MediaAnalyzer;
use photos_core::{get_relative_path_str, store_media_item};
use ruurd_photos_thumbnail_generation::{generate_thumbnails, ThumbOptions};
use sqlx::PgTransaction;
use std::path::Path;

pub async fn process_file(
    file: &Path,
    config: &ThumbOptions,
    analyzer: &mut MediaAnalyzer,
    tx: &mut PgTransaction<'_>,
) -> color_eyre::Result<()> {
    let relative_path_str = get_relative_path_str(file)?;
    let tiny_thumb_path = config
        .thumbnails_dir
        .join(file.file_name().unwrap())
        .join("10p.avif");

    generate_thumbnails(&file, config).await?;

    let media_info = analyzer.analyze_media(&file, &tiny_thumb_path).await?;

    store_media_item(tx, &relative_path_str, &media_info).await?;

    Ok(())
}
