use std::sync::OnceLock;

use crate::Config;
use ruurd_photos_thumbnail_generation::ThumbOptions;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf, absolute};

static CONFIG: OnceLock<Config> = OnceLock::new();
static THUMBNAIL_OPTIONS: OnceLock<ThumbOptions> = OnceLock::new();

/// Retrieves the application configuration, loading it from `config/config.yaml` if not already loaded.
///
/// # Panics
/// * If `config/config.yaml` is not found.
/// * If there's an error parsing `config.yaml`.
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config_str =
            fs::read_to_string("config/config.yaml").expect("config/config.yaml not found");

        serde_yaml::from_str(&config_str).expect("Error reading config.yaml file")
    })
}

/// Returns the absolute path to the directory where thumbnails are stored.
///
/// # Panics
/// * If the configured thumbnails directory path is invalid.
#[must_use]
pub fn get_thumbnails_dir() -> PathBuf {
    let thumb_dir = &get_config().directories.thumbnails_folder;
    absolute(Path::new(thumb_dir)).expect("Invalid thumbnails dir")
}

/// Returns the absolute path to the main media directory.
///
/// # Panics
/// * If the configured media directory path is invalid.
#[must_use]
pub fn get_media_dir() -> PathBuf {
    let media_dir = &get_config().directories.media_folder;
    absolute(media_dir).expect("invalid media dir")
}

/// Retrieves the thumbnail generation options, initializing them from the application configuration if not already loaded.
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
