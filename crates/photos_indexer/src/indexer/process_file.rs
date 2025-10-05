use crate::database::insert_media_item::insert_full_media_item;
use media_analyzer::MediaAnalyzer;
use ruurd_photos_thumbnail_generation::{generate_thumbnails, ThumbOptions};
use sqlx::PgPool;
use std::path::Path;

pub async fn process_file(
    media_dir: &Path,
    file: &Path,
    config: &ThumbOptions,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> color_eyre::Result<()> {
    println!("Processing {}", file.display());

    let file = file.canonicalize()?;
    let media_dir = media_dir.canonicalize()?;

    generate_thumbnails(&file, config).await?;
    
    let thumb_path = config
        .thumbnails_dir
        .join(file.file_name().unwrap())
        .join("10p.avif");
    let media_info = analyzer.analyze_media(&file, &thumb_path).await?;
    let relative_path = file.strip_prefix(media_dir)?;
    let relative_path_str = relative_path.to_string_lossy().to_string();
    insert_full_media_item(pool, &relative_path_str, &media_info).await?;
    Ok(())
}

pub async fn remove_file(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let _file = file;
    let _pool = pool;
    Ok(())
}
