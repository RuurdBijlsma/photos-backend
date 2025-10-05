use ruurd_photos_thumbnail_generation::{ThumbOptions, VideoOutputFormat};
use std::path::{Path, PathBuf};

pub fn get_thumbnails_dir() -> PathBuf {
    Path::new("thumbnails").to_path_buf()
}

pub fn get_media_dir() -> PathBuf {
    Path::new("assets").canonicalize().expect("Media dir")
}

pub fn get_thumbnail_options(thumbs_dir: &Path) -> ThumbOptions {
    ThumbOptions {
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
    }
}
