use crate::db_helpers::write_to_db::store_media_item;
use common_photos::{
    get_config, get_relative_path_str, get_thumbnail_options, get_thumbnails_dir, nice_id,
};
use media_analyzer::MediaAnalyzer;
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
    let media_item_id = nice_id(get_config().database.media_item_id_length);
    let thumbnail_out_dir = thumb_base_dir.join(&media_item_id);
    let smallest_thumb_size = thumb_config
        .heights
        .iter()
        .min()
        .expect("Thumb config heights should have at least one item.");
    let smallest_thumb_filename = format!("{}p.avif", smallest_thumb_size);
    let tiny_thumb_path = thumbnail_out_dir.join(smallest_thumb_filename);

    generate_thumbnails(file, &thumbnail_out_dir, thumb_config).await?;

    let media_info = analyzer.analyze_media(file, &tiny_thumb_path).await?;

    store_media_item(tx, &relative_path_str, &media_info, &media_item_id).await?;

    Ok(())
}
