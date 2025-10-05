use photos_core::enqueue_file;
use ruurd_photos_thumbnail_generation::ThumbOptions;
use sqlx::{Pool, Postgres};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn get_media_files(folder: &Path, allowed_exts: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(folder)
        .into_iter()
        .filter_map(color_eyre::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| allowed_exts.contains(&ext.to_lowercase().as_str()))
        })
        .map(|e| e.into_path())
        .collect()
}

pub async fn scan_all_files(
    media_dir: &Path,
    config: &ThumbOptions,
    pool: &Pool<Postgres>,
) -> color_eyre::Result<()> {
    let mut all_files = get_media_files(
        media_dir,
        &config
            .photo_extensions
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );
    all_files.extend(get_media_files(
        media_dir,
        &config
            .video_extensions
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    ));

    for file in all_files {
        enqueue_file(&file, pool).await?;
    }

    Ok(())
}
