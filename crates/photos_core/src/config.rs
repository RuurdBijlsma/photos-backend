use ruurd_photos_thumbnail_generation::{ThumbOptions, VideoOutputFormat};
use std::path::{absolute, Path, PathBuf};

#[derive(Debug)]
pub struct WorkerConfig {
    pub max_retries: i32,
    pub wait_after_empty_queue_s: u64,
    pub wait_after_error_s: u64,
}

pub fn worker_config() -> WorkerConfig {
    WorkerConfig {
        wait_after_empty_queue_s: 10,
        wait_after_error_s: 5,
        max_retries: 5,
    }
}

pub fn media_item_id_length() -> usize {
    10
}

pub fn get_thumbnails_dir() -> PathBuf {
    absolute(Path::new("thumbnails")).expect("Invalid thumbnails dir")
}

pub fn get_media_dir() -> PathBuf {
    let media_dir = Path::new("assets");
    absolute(media_dir).expect("invalid media dir")
}

pub fn get_thumbnail_options() -> ThumbOptions {
    ThumbOptions {
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
    }
}
