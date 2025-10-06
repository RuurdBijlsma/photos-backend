use crate::db_helpers::write_to_db::store_media_item;
use media_analyzer::MediaAnalyzer;
use common_photos::{
    get_relative_path_str, get_thumbnail_options, get_thumbnails_dir, media_item_id_length, nice_id,
};
use ruurd_photos_thumbnail_generation::generate_thumbnails;
use sqlx::PgTransaction;
use std::path::Path;

pub async fn ingest_file(
    file: &Path,
    analyzer: &mut MediaAnalyzer,
    tx: &mut PgTransaction<'_>,
) -> color_eyre::Result<()> {
    let thumb_config = get_thumbnail_options();
    let relative_path_str = get_relative_path_str(file)?;
    let thumb_base_dir = get_thumbnails_dir();
    let media_item_id = nice_id(media_item_id_length());
    let thumbnail_out_dir = thumb_base_dir.join(&media_item_id);
    let tiny_thumb_path = thumbnail_out_dir.join("10p.avif");

    generate_thumbnails(file, &thumbnail_out_dir, &thumb_config).await?;

    let media_info = analyzer.analyze_media(file, &tiny_thumb_path).await?;

    store_media_item(tx, &relative_path_str, &media_info, &media_item_id).await?;

    Ok(())
}
