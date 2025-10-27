use crate::thumbnails::ffmpeg_photo_thumbnail::generate_ffmpeg_photo_thumbnails;
use crate::thumbnails::photo_thumbnails::generate_photo_thumbnails;
use crate::thumbnails::video_thumbnails::generate_video_thumbnails;
use crate::utils::move_dir_contents;
use color_eyre::Result;
use common_photos::{is_photo_file, is_video_file, ThumbOptions};
use std::fs;
use std::path::Path;
use temp_dir::TempDir;

fn thumbs_exist(file: &Path, thumb_folder: &Path, config: &ThumbOptions) -> Result<bool> {
    let is_photo= is_photo_file(file);
    let is_video= is_video_file(file);
    let photo_thumb_ext = &config.thumbnail_extension;
    let video_thumb_ext = &config.video_options.extension;
    let mut should_exist: Vec<String> = vec![];

    if is_photo || is_video {
        // Both photo and video should have a thumbnail for each entry in .heights.
        for h in &config.heights {
            should_exist.push(format!("{h}p.{photo_thumb_ext}"));
        }
    }
    if is_video {
        for p in &config.video_options.percentages {
            should_exist.push(format!("{p}_percent.{photo_thumb_ext}"));
        }
        for x in &config.video_options.transcode_outputs {
            let height = x.height;
            should_exist.push(format!("{height}p.{video_thumb_ext}"));
        }
    }

    for thumb_filename in should_exist {
        if !fs::exists(thumb_folder.join(thumb_filename.clone()))? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Generates thumbnails for a given media file (image or video) based on the provided configuration.
///
/// This function detects the file type based on its extension and then calls the appropriate
/// thumbnail generation logic.
///
/// - For supported image types, it generates resized thumbnails.
/// - For supported video types, it can generate a complex combination of still images and video previews.
///
/// The generated files are first created in a temporary directory and then moved to a dedicated
/// subfolder within the `thumbs_dir`, named after the original file.
///
/// # Arguments
///
/// * `file` - The path to the source image or video file.
/// * `out_folder` - Where to output the thumbnail files.
/// * `config` - An `ThumbOptions` struct detailing what thumbnails to generate.
///
/// # Errors
///
/// This function will return an error if:
/// - File paths are invalid.
/// - The `ffmpeg` or `ffprobe` commands fail.
/// - There are issues with file I/O, such as creating directories or moving files.
pub async fn generate_thumbnails(
    file: &Path,
    out_folder: &Path,
    config: &ThumbOptions,
    orientation: u8,
) -> Result<()> {
    let Some(extension) = file.extension().and_then(|s| s.to_str()) else {
        return Ok(());
    };

    if config.skip_if_exists && thumbs_exist(file, out_folder, config)? {
        return Ok(());
    }

    let extension = extension.to_lowercase();
    let temp_dir = TempDir::new()?;
    let temp_out_dir = temp_dir.path();

    if config.photo_extensions.contains(&extension) {
        if config.thumbnail_extension == "avif" {
            generate_photo_thumbnails(file, temp_out_dir, config, orientation)?;
        } else {
            generate_ffmpeg_photo_thumbnails(file, temp_out_dir, config).await?;
        }
    } else if config.video_extensions.contains(&extension) {
        generate_video_thumbnails(file, temp_out_dir, config).await?;
    }

    move_dir_contents(temp_out_dir, out_folder).await?;
    temp_dir.cleanup()?;

    Ok(())
}
