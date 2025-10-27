use crate::ffmpeg::FfmpegCommand;
use color_eyre::eyre::{eyre, Result};
use common_photos::ThumbOptions;
use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};
use image::{ImageBuffer, ImageReader, Rgba};
use imgref::Img;
use ravif::Encoder;
use rayon::prelude::*;
use rgb::RGBA;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

/// Generates photo thumbnails using a native Rust image processing library.
/// This is optimized for AVIF output and supports EXIF orientation correction.
pub fn generate_native_photo_thumbnails(
    input_path: &Path,
    output_dir: &Path,
    config: &ThumbOptions,
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
    img = match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    };

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
            let mut resizer = Resizer::new();
            let encoder = Encoder::new()
                .with_quality(config.avif_options.quality)
                .with_speed(config.avif_options.speed)
                .with_alpha_quality(config.avif_options.alpha_quality);

            let target_w = ((u64::from(orig_w) * target_h) / u64::from(orig_h)) as u32;
            if target_w == 0 || target_h == 0 {
                return Ok(());
            }

            let mut dst_img = Image::new(
                NonZeroU32::new(target_w).unwrap().into(),
                NonZeroU32::new(target_h as u32).unwrap().into(),
                PixelType::U8x4,
            );

            resizer.resize(&src_image, &mut dst_img, None)?;

            let resized_img =
                ImageBuffer::<Rgba<u8>, _>::from_raw(target_w, target_h as u32, dst_img.into_vec())
                    .ok_or_else(|| eyre!("Failed to construct resized image"))?;

            let rgba_vec: Vec<RGBA<u8>> = resized_img
                .pixels()
                .map(|p| RGBA {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: p[3],
                })
                .collect();
            let img_ref = Img::new(&rgba_vec[..], target_w as usize, target_h as usize);
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
    config: &ThumbOptions,
) -> Result<()> {
    if config.heights.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(output_dir)?;

    let mut cmd = FfmpegCommand::new(input);
    let input_stream = "[0:v]";
    let output_streams = cmd.add_split(input_stream, config.heights.len());

    for (i, &h) in config.heights.iter().enumerate() {
        let scaled_stream = cmd.add_scale(&output_streams[i], -1, h as i32);
        let out_path = output_dir.join(format!("{h}p.{}", config.thumbnail_extension));
        cmd.map_still_output(&scaled_stream, &out_path);
    }

    cmd.run().await
}