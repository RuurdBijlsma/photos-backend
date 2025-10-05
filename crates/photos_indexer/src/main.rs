mod database;
mod indexer;

use color_eyre::Result;
use photos_core::get_db_pool;
use std::path::Path;
use ruurd_photos_thumbnail_generation::{ThumbOptions, VideoOutputFormat};
use tokio::fs;
use crate::indexer::scan_all::scan_all_files;
use crate::indexer::watcher::start_watching;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let pool = get_db_pool().await?;

    let source_folder = Path::new("assets");
    let thumbs_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbs_dir).await?;


    let config = ThumbOptions {
        thumbnails_dir: thumbs_dir.to_path_buf(),
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

    println!("Scanning {} ...", source_folder.display());
    scan_all_files(source_folder, &config, &pool).await?;
    println!("Scan done, start watching for file changes...");
    start_watching(source_folder, &config, &pool)?;

    Ok(())
}
