mod database;

use crate::database::insert_media_item::insert_full_media_item;
use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use ruurd_photos_thumbnail_generation::{generate_thumbnails, ThumbOptions, VideoOutputFormat};
use sqlx::PgPool;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;
use walkdir::WalkDir;

fn get_media_files(folder: &Path, allowed_exts: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(folder)
        .into_iter()
        .filter_map(Result::ok)
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

async fn process_file(
    file: &Path,
    thumbs_dir: &Path,
    config: &ThumbOptions,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> Result<()> {
    println!("Processing {}", file.display());
    generate_thumbnails(file, thumbs_dir, config).await?;
    let thumb_path = thumbs_dir.join(file.file_name().unwrap()).join("10p.avif");
    let media_info = analyzer.analyze_media(file, &thumb_path).await?;
    insert_full_media_item(pool, file.to_str().unwrap(), &media_info).await?;
    Ok(())
}d

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::from_path(".env").ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    photos_core::add(1,2);

    let source_folder = Path::new("assets");
    let thumbs_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbs_dir).await?;

    let analyzer = Arc::new(Mutex::new(MediaAnalyzer::builder().build().await?));

    let config = ThumbOptions {
        thumb_ext: "avif".to_string(),
        transcode_ext: "webm".to_string(),
        skip_if_exists: true,
        video_extensions: [
            "mp4", "webm", "av1", "3gp", "mov", "mkv", "flv", "m4v", "m4p",
        ]
        .map(String::from)
        .to_vec(),
        photo_extensions: ["jpg", "jpeg", "png", "gif", "tiff", "tga", "avif"]
            .map(String::from)
            .to_vec(),
        heights: vec![10, 240, 480, 1080],
        thumb_time: 0.5,
        percentages: vec![0, 33, 66, 99],
        height: 720,
        output_videos: vec![
            VideoOutputFormat {
                height: 480,
                quality: 35,
            },
            VideoOutputFormat {
                height: 144,
                quality: 40,
            },
        ],
    };

    let mut all_files = get_media_files(
        source_folder,
        &config
            .photo_extensions
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );
    all_files.extend(get_media_files(
        source_folder,
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
            process_file(&file, thumbs_dir, &config, &mut guard, &pool).await
        };

        if let Err(e) = Retry::spawn(retry_strategy.clone(), action).await {
            eprintln!("Failed to process {}: {}", file.display(), e);
        }
    }

    Ok(())
}
