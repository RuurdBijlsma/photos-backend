use crate::db_helpers::write_to_db::store_media_item;
use crate::utils::get_thumb_options;
use common_photos::{nice_id, relative_path_no_exist, settings, thumbnails_dir};
use media_analyzer::MediaAnalyzer;
use ruurd_photos_thumbnail_generation::generate_thumbnails;
use sqlx::PgTransaction;
use std::path::Path;
use std::time::Instant;

/// Processes a media file by generating thumbnails, analyzing its metadata, and storing the result.
///
/// # Errors
///
/// * Fails if any of the following operations return an error:
///   - Path resolution (`get_relative_path_str`)
///   - Thumbnail generation (`generate_thumbnails`)
///   - Media analysis (`analyzer.analyze_media`)
///   - Database insertion (`store_media_item`)
///
/// # Panics
///
/// * Panics if the `thumbnail_generation.heights` configuration array is empty.
pub async fn ingest_file(
    file: &Path,
    user_id: i32,
    analyzer: &mut MediaAnalyzer,
    tx: &mut PgTransaction<'_>,
) -> color_eyre::Result<()> {
    let now = Instant::now();
    let thumb_config = get_thumb_options();
    let relative_path_str = relative_path_no_exist(file)?;
    let thumb_base_dir = thumbnails_dir();
    let media_item_id = nice_id(settings().database.media_item_id_length);
    let thumbnail_out_dir = thumb_base_dir.join(&media_item_id);
    let smallest_thumb_size = thumb_config
        .heights
        .iter()
        .min()
        .expect("Thumb config heights should have at least one item.");
    let smallest_thumb_filename = format!("{smallest_thumb_size}p.avif");
    let tiny_thumb_path = thumbnail_out_dir.join(smallest_thumb_filename);
    println!("setup: {:?}", now.elapsed());

    let now = Instant::now();
    generate_thumbnails(file, &thumbnail_out_dir, &thumb_config).await?;
    println!("generate_thumbnails: {:?}", now.elapsed());

    let now = Instant::now();
    let media_info = analyzer.analyze_media(file, &tiny_thumb_path).await?;
    println!("analyze_media: {:?}", now.elapsed());

    let now = Instant::now();
    store_media_item(tx, &relative_path_str, &media_info, &media_item_id, user_id).await?;
    println!("store_media_item: {:?}", now.elapsed());

    Ok(())
}
