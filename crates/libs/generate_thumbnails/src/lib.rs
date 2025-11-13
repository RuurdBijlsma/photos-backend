#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

//! # Thumbnail Generation Crate
//!
//! A Rust crate for generating a variety of thumbnails for photos and videos.
//!
//! This library provides a flexible and efficient way to create thumbnails using either
//! native Rust image processing for formats like AVIF or by leveraging the power of `FFmpeg`
//! for a wide range of media types.
//!
//! ## Features
//!
//! - **Photo Thumbnails**: Generate high-quality, multi-sized thumbnails from source images.
//!   - Supports fast, native AVIF encoding.
//!   - Can fall back to `FFmpeg` for other formats.
//!   - Automatic EXIF orientation correction.
//! - **Video Thumbnails**: Create a comprehensive set of thumbnails from videos.
//!   - Generate still frames at specific timestamps or percentages.
//!   - Create transcoded video previews in different resolutions (e.g., 480p, 720p `WebM`).
//! - **Efficient Processing**:
//!   - Uses a single `FFmpeg` command to generate all required outputs simultaneously, minimizing process overhead.
//!   - Leverages parallel processing for native image resizing.
//! - **Flexible Configuration**: A comprehensive `ThumbOptions` struct allows for detailed control over the output.
//!
//! ## Entry Points
//!
//! The two main entry points for this crate are:
//!
//! - `generate_thumbnails`: The primary function to generate all configured thumbnails for a given file.
//! - `thumbs_exist`: A utility function to check if all expected thumbnails for a file already exist.

mod ffmpeg;
mod photo;
mod utils;
mod video;

use color_eyre::Result;
use common_services::settings::ThumbOptions;
use common_services::utils::{is_photo_file, is_video_file};
use std::path::Path;
use temp_dir::TempDir;

/// Checks if all the configured thumbnails for a given media file already exist.
///
/// This function is useful for skipping regeneration of thumbnails that are already present.
/// It constructs the expected filenames for both photo and video thumbnails based on the
/// provided configuration and checks for their existence in the specified `thumb_folder`.
///
/// # Arguments
///
/// * `file` - Path to the source media file.
/// * `thumb_folder` - The directory where the thumbnails are expected to be.
/// * `config` - A `ThumbOptions` struct detailing what thumbnails should exist.
///
/// # Returns
///
/// Returns `Ok(true)` if all expected thumbnails exist, `Ok(false)` otherwise.
/// Returns an error if there's an issue accessing the filesystem.
#[must_use]
pub fn thumbs_exist(file: &Path, thumb_folder: &Path, config: &ThumbOptions) -> bool {
    let photo_thumb_ext = &config.thumbnail_extension;
    let video_thumb_ext = &config.video_options.extension;

    let is_photo = is_photo_file(file);
    let is_video = is_video_file(file);

    let photo_stills_exist = || {
        config.heights.iter().all(|h| {
            let thumb_path = thumb_folder.join(format!("{h}p.{photo_thumb_ext}"));
            thumb_path.exists()
        })
    };

    let video_stills_exist = || {
        config.video_options.percentages.iter().all(|p| {
            let thumb_path = thumb_folder.join(format!("{p}_percent.{photo_thumb_ext}"));
            thumb_path.exists()
        })
    };

    let video_transcodes_exist = || {
        config.video_options.transcode_outputs.iter().all(|x| {
            let thumb_path = thumb_folder.join(format!("{}p.{}", x.height, video_thumb_ext));
            thumb_path.exists()
        })
    };

    if (is_photo || is_video) && !photo_stills_exist() {
        return false;
    }

    if is_video && (!video_stills_exist() || !video_transcodes_exist()) {
        return false;
    }

    true
}

/// Generates thumbnails for a given media file (image or video) based on the provided configuration.
///
/// This function detects the file type and calls the appropriate thumbnail generation logic.
/// Generated files are created in a temporary directory and then moved to their final destination.
///
/// # Arguments
///
/// * `file` - The path to the source image or video file.
/// * `out_folder` - Where to output the thumbnail files.
/// * `config` - An `ThumbOptions` struct detailing what thumbnails to generate.
/// * `orientation` - The EXIF orientation value (1-8) for photos.
///
/// # Errors
///
/// Returns an error if paths are invalid, `FFmpeg` commands fail, or file I/O operations fail.
pub async fn generate_thumbnails(
    file: &Path,
    out_folder: &Path,
    config: &ThumbOptions,
    orientation: Option<u64>,
) -> Result<()> {
    let Some(extension) = file.extension().and_then(|s| s.to_str()) else {
        return Ok(());
    };
    let orientation = orientation.unwrap_or(0);

    if config.skip_if_exists && thumbs_exist(file, out_folder, config) {
        return Ok(());
    }

    let extension = extension.to_lowercase();
    let temp_dir = TempDir::new()?;
    let temp_out_dir = temp_dir.path();

    if config.photo_extensions.contains(&extension) {
        if config.thumbnail_extension == "avif" {
            photo::generate_native_photo_thumbnails(file, temp_out_dir, config, orientation)?;
        } else {
            photo::generate_ffmpeg_photo_thumbnails(file, temp_out_dir, config).await?;
        }
    } else if config.video_extensions.contains(&extension) {
        video::generate_video_thumbnails(file, temp_out_dir, config).await?;
    }

    utils::move_dir_contents(temp_out_dir, out_folder).await?;
    temp_dir.cleanup()?;

    Ok(())
}
