use crate::QualityData;
use color_eyre::eyre::Result;
use image::{imageops, GrayImage, DynamicImage};
use imageproc::filter::{laplacian_filter, median_filter};
use std::path::Path;

pub fn get_quality_data(image_path: &Path) -> Result<QualityData> {
    let img = image::ImageReader::open(image_path)?.decode()?;
    let img = resize_if_large(img, 1024);
    let gray_img = img.to_luma8();

    let texture = calculate_texture(&gray_img);

    let blurriness = calculate_blurriness(&gray_img, texture);
    let noisiness = calculate_noise(&gray_img, texture);
    let exposure = calculate_exposure(&gray_img);

    let final_score = (blurriness * 0.5 + noisiness * 0.3 + exposure * 0.2)
        / (0.5 + 0.3 + 0.2)
        * 100.0;

    Ok(QualityData {
        quality_score: final_score,
        blurriness,
        noisiness,
        exposure,
    })
}

pub(crate) fn resize_if_large(img: DynamicImage, max_dim: u32) -> DynamicImage {
    if img.width() > max_dim || img.height() > max_dim {
        img.resize(max_dim, max_dim, imageops::FilterType::Lanczos3)
    } else {
        img
    }
}

// RMS contrast
fn calculate_contrast(gray_img: &GrayImage) -> f64 {
    let n = (gray_img.width() * gray_img.height()) as f64;
    if n == 0.0 { return 0.0; }

    let sum: f64 = gray_img.pixels().map(|p| p[0] as f64).sum();
    let mean = sum / n;

    let sum_sq_diff: f64 = gray_img
        .pixels()
        .map(|p| {
            let diff = p[0] as f64 - mean;
            diff * diff
        })
        .sum();

    (sum_sq_diff / n).sqrt() / 255.0
}

fn calculate_texture(gray_img: &GrayImage) -> f64 {
    let w = gray_img.width() as usize;
    let h = gray_img.height() as usize;
    let window = 5;
    let half = window / 2;

    // Create integral images for sum and sum of squares
    let mut sum = vec![vec![0f64; w + 1]; h + 1];
    let mut sum_sq = vec![vec![0f64; w + 1]; h + 1];

    for y in 0..h {
        let mut row_sum = 0.0;
        let mut row_sum_sq = 0.0;
        for x in 0..w {
            let val = gray_img.get_pixel(x as u32, y as u32)[0] as f64;
            row_sum += val;
            row_sum_sq += val * val;
            sum[y + 1][x + 1] = sum[y][x + 1] + row_sum;
            sum_sq[y + 1][x + 1] = sum_sq[y][x + 1] + row_sum_sq;
        }
    }

    let mut total_sd = 0.0;
    let mut count = 0.0;

    for y in half..(h - half) {
        for x in half..(w - half) {
            let x0 = x - half;
            let y0 = y - half;
            let x1 = x + half + 1;
            let y1 = y + half + 1;
            let n = (window * window) as f64;

            let sum_window = sum[y1][x1] + sum[y0][x0] - sum[y1][x0] - sum[y0][x1];
            let sum_sq_window = sum_sq[y1][x1] + sum_sq[y0][x0] - sum_sq[y1][x0] - sum_sq[y0][x1];

            let mean = sum_window / n;
            let sd = ((sum_sq_window / n) - mean * mean).max(0.0).sqrt();
            total_sd += sd;
            count += 1.0;
        }
    }

    (total_sd / count) / 128.0 // normalize to [0,1]
}


// Blurriness: Laplacian variance scaled by contrast and reduced by texture
fn calculate_blurriness(gray_img: &GrayImage, texture: f64) -> f64 {
    let lap = laplacian_filter(gray_img);
    let n = (lap.width() * lap.height()) as f64;
    if n < 2.0 { return 0.0; }

    let mut sum = 0.0;
    let mut sum_sq = 0.0;

    for p in lap.pixels() {
        let v = p[0] as f64;
        sum += v;
        sum_sq += v * v;
    }

    let mean = sum / n;
    let variance = (sum_sq - n * mean * mean) / (n - 1.0);

    let img_contrast = calculate_contrast(gray_img);
    let adjusted_var = variance * img_contrast * (1.0 - 0.5 * texture); // texture reduces penalty

    match adjusted_var {
        v if v <= 50.0 => 0.0,
        v if v >= 1000.0 => 1.0,
        v => (v - 50.0) / 950.0,
    }
}

// Noise: median filter difference reduced by texture
fn calculate_noise(gray_img: &GrayImage, texture: f64) -> f64 {
    let denoised = median_filter(gray_img, 5, 5);
    let n = (gray_img.width() * gray_img.height()) as f64;
    let mut sum_diff = 0.0;

    for (p_orig, p_denoised) in gray_img.pixels().zip(denoised.pixels()) {
        sum_diff += (p_orig[0] as f64 - p_denoised[0] as f64).abs();
    }

    let mean_diff = sum_diff / n;
    let raw_noise = match mean_diff {
        d if d <= 2.0 => 1.0,
        d if d >= 15.0 => 0.0,
        d => 1.0 - (d - 2.0) / 13.0,
    };

    raw_noise * (1.0 - 0.5 * texture)
}

// Exposure: histogram clipping
fn calculate_exposure(gray_img: &GrayImage) -> f64 {
    let mut histogram = [0u32; 256];
    for p in gray_img.pixels() {
        histogram[p[0] as usize] += 1;
    }

    let n = (gray_img.width() * gray_img.height()) as f64;
    let mut clipped = 0.0;
    for i in 0..256 {
        if i < 40 || i > 215 {
            clipped += histogram[i] as f64;
        }
    }

    let exposure_goodness = 1.0 - (clipped / n);
    exposure_goodness.clamp(0.0, 1.0)
}
