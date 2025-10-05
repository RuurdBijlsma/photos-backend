use crate::indexer::process_file::process_file;
use media_analyzer::MediaAnalyzer;
use ruurd_photos_thumbnail_generation::ThumbOptions;
use sqlx::{Pool, Postgres};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;
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

pub async fn scan_all_files(media_dir: &Path, config: &ThumbOptions, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    let analyzer = Arc::new(Mutex::new(MediaAnalyzer::builder().build().await?));

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

    let retry_strategy = FixedInterval::from_millis(200).take(3);

    for file in all_files {
        let analyzer = Arc::clone(&analyzer);
        let action = || async {
            let mut guard = analyzer.lock().await;
            process_file(&file, config, &mut guard, pool).await
        };

        if let Err(e) = Retry::spawn(retry_strategy.clone(), action).await {
            eprintln!("Failed to process {}: {}", file.display(), e);
        }
    }

    Ok(())
}