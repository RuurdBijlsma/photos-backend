use std::sync::OnceLock;

use ruurd_photos_thumbnail_generation::ThumbOptions;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf, absolute};
use crate::Config;

static CONFIG: OnceLock<Config> = OnceLock::new();
static THUMBNAIL_OPTIONS: OnceLock<ThumbOptions> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config_str = fs::read_to_string("../../../../config/config.yaml")
            .expect("config/indexer_config.yaml not found");

        serde_yaml::from_str(&config_str).expect("Error reading indexer_config.yaml file")
    })
}

pub fn get_thumbnails_dir() -> PathBuf {
    let thumb_dir = &get_config().directories.thumbnails_folder;
    absolute(Path::new(thumb_dir)).expect("Invalid thumbnails dir")
}

pub fn get_media_dir() -> PathBuf {
    let media_dir = &get_config().directories.media_folder;
    absolute(media_dir).expect("invalid media dir")
}

pub fn get_thumbnail_options() -> &'static ThumbOptions {
    THUMBNAIL_OPTIONS.get_or_init(|| {
        let thumb_gen_config = &get_config().thumbnail_generation;
        ThumbOptions {
            video_options: thumb_gen_config.video_options.clone(),
            avif_options: thumb_gen_config.avif_options.clone(),
            heights: thumb_gen_config.heights.clone(),
            thumbnail_extension: thumb_gen_config.thumbnail_extension.clone(),
            photo_extensions: thumb_gen_config.photo_extensions.clone(),
            video_extensions: thumb_gen_config.video_extensions.clone(),
            skip_if_exists: true,
        }
    })
}
