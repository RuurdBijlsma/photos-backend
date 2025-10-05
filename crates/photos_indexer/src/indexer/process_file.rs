use crate::database::insert_media_item::insert_full_media_item;
use media_analyzer::MediaAnalyzer;
use ruurd_photos_thumbnail_generation::{generate_thumbnails, ThumbOptions};
use sqlx::PgPool;
use std::path::Path;

pub async fn process_file(
    file: &Path,
    config: &ThumbOptions,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> color_eyre::Result<()> {
    println!("Processing {}", file.display());
    generate_thumbnails(file, config).await?;
    let thumb_path = config
        .thumbnails_dir
        .join(file.file_name().unwrap())
        .join("10p.avif");
    let media_info = analyzer.analyze_media(file, &thumb_path).await?;
    insert_full_media_item(pool, file.to_str().unwrap(), &media_info).await?;
    Ok(())
}
