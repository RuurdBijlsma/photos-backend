use crate::ffmpeg::FfmpegCommand;
use app_state::ThumbnailSettings;
use color_eyre::eyre::{Result, eyre};
use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};
use image::ImageReader;
use image::metadata::Orientation;
use imgref::Img;
use ravif::Encoder;
use rayon::prelude::*;
use rgb::FromSlice;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

/// Generates photo thumbnails using a native Rust image processing library.
/// This is optimized for AVIF output and supports EXIF orientation correction.
pub fn generate_native_photo_thumbnails(
    input_path: &Path,
    output_dir: &Path,
    config: &ThumbnailSettings,
    orientation: u64,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    if config.heights.is_empty() {
        return Ok(());
    }

    let mut img = ImageReader::open(input_path)?
        .with_guessed_format()?
        .decode()?;

    // Correct the orientation based on the EXIF data.
    let image_orientation = match orientation {
        2 => Orientation::FlipHorizontal,
        3 => Orientation::Rotate180,
        4 => Orientation::FlipVertical,
        5 => Orientation::Rotate90FlipH,
        6 => Orientation::Rotate90,
        7 => Orientation::Rotate270FlipH,
        8 => Orientation::Rotate270,
        _ => Orientation::NoTransforms,
    };
    img.apply_orientation(image_orientation);

    let src_img = img.into_rgba8();
    let (orig_w, orig_h) = src_img.dimensions();

    let src_image = Image::from_vec_u8(
        NonZeroU32::new(orig_w)
            .ok_or_else(|| eyre!("source image width is zero"))?
            .into(),
        NonZeroU32::new(orig_h)
            .ok_or_else(|| eyre!("source image height is zero"))?
            .into(),
        src_img.into_raw(),
        PixelType::U8x4,
    )?;

    config
        .heights
        .par_iter()
        .try_for_each(|&target_h| -> Result<()> {
            let mut target_w = ((u64::from(orig_w) * target_h) / u64::from(orig_h)) as u32;
            if target_w > 0 && !target_w.is_multiple_of(2) {
                target_w += 1;
            }
            let mut dst_img = Image::new(target_w, target_h as u32, PixelType::U8x4);
            let mut r = Resizer::new();
            r.resize(&src_image, &mut dst_img, None)?;
            let raw_pixels = dst_img.buffer();
            let rgba_pixels = raw_pixels.as_rgba();
            let img_ref = Img::new(rgba_pixels, target_w as usize, target_h as usize);
            let encoder = Encoder::new()
                .with_quality(config.avif_options.quality)
                .with_speed(config.avif_options.speed)
                .with_alpha_quality(config.avif_options.alpha_quality);
            let avif_data = encoder.encode_rgba(img_ref)?;
            fs::write(
                output_dir.join(format!("{target_h}p.avif")),
                avif_data.avif_file,
            )?;

            Ok(())
        })?;

    Ok(())
}

/// Generates photo thumbnails using `FFmpeg`.
pub async fn generate_ffmpeg_photo_thumbnails(
    input: &Path,
    output_dir: &Path,
    config: &ThumbnailSettings,
) -> Result<()> {
    if config.heights.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(output_dir)?;

    let mut cmd = FfmpegCommand::new(input);
    let input_stream = "[0:v]";
    let output_streams = cmd.add_split(input_stream, config.heights.len());

    for (i, &h) in config.heights.iter().enumerate() {
        // -2 logic ensures even width
        let scaled_stream = cmd.add_scale(&output_streams[i], -2, h as i32);
        let out_path = output_dir.join(format!("{h}p.{}", config.thumbnail_extension));
        cmd.map_still_output(&scaled_stream, &out_path);
    }

    cmd.run().await
}
