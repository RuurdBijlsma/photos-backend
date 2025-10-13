use crate::QualityData;
use color_eyre::eyre::Result;
use image::{DynamicImage, GrayImage, imageops};
use imageproc::filter::{laplacian_filter, median_filter};
use std::path::Path;

pub fn get_quality_data(image_path: &Path) -> Result<QualityData> {
    let img = image::ImageReader::open(image_path)?.decode()?;
    let gray_img = resize_if_large(img, 1024).to_luma8();

    let texture = calculate_texture(&gray_img);
    let blurriness = calculate_blurriness(&gray_img, texture);
    let noisiness = calculate_noise(&gray_img, texture);
    let exposure = calculate_exposure(&gray_img);

    let final_score = (blurriness * 0.5 + noisiness * 0.3 + exposure * 0.2) / 1.0 * 100.0;

    Ok(QualityData {
        quality_score: final_score,
        blurriness,
        noisiness,
        exposure,
    })
}

fn resize_if_large(img: DynamicImage, max_dim: u32) -> DynamicImage {
    if img.width() > max_dim || img.height() > max_dim {
        img.resize(max_dim, max_dim, imageops::FilterType::Lanczos3)
    } else {
        img
    }
}

fn calculate_contrast(gray_img: &GrayImage) -> f64 {
    let n = (gray_img.width() * gray_img.height()) as f64;
    if n == 0.0 {
        return 0.0;
    }

    let mean = gray_img.pixels().map(|p| p[0] as f64).sum::<f64>() / n;
    ((gray_img
        .pixels()
        .map(|p| (p[0] as f64 - mean).powi(2))
        .sum::<f64>()
        / n)
        .sqrt())
        / 255.0
}

fn calculate_texture(gray_img: &GrayImage) -> f64 {
    let (w, h) = (gray_img.width() as usize, gray_img.height() as usize);
    let window = 5;
    let half = window / 2;

    let mut sum = vec![0f64; (w + 1) * (h + 1)];
    let mut sum_sq = vec![0f64; (w + 1) * (h + 1)];

    let idx = |x: usize, y: usize| y * (w + 1) + x;

    for y in 0..h {
        let mut row_sum = 0.0;
        let mut row_sum_sq = 0.0;
        for x in 0..w {
            let val = gray_img.get_pixel(x as u32, y as u32)[0] as f64;
            row_sum += val;
            row_sum_sq += val * val;

            sum[idx(x + 1, y + 1)] = sum[idx(x + 1, y)] + row_sum;
            sum_sq[idx(x + 1, y + 1)] = sum_sq[idx(x + 1, y)] + row_sum_sq;
        }
    }

    let mut total_sd = 0.0;
    let mut count = 0.0;

    for y in half..(h - half) {
        for x in half..(w - half) {
            let (x0, y0) = (x - half, y - half);
            let (x1, y1) = (x + half + 1, y + half + 1);
            let n = (window * window) as f64;

            let sum_w = sum[idx(x1, y1)] + sum[idx(x0, y0)] - sum[idx(x1, y0)] - sum[idx(x0, y1)];
            let sum_sq_w = sum_sq[idx(x1, y1)] + sum_sq[idx(x0, y0)]
                - sum_sq[idx(x1, y0)]
                - sum_sq[idx(x0, y1)];

            let mean = sum_w / n;
            total_sd += ((sum_sq_w / n) - mean * mean).max(0.0).sqrt();
            count += 1.0;
        }
    }

    (total_sd / count) / 128.0
}

fn calculate_blurriness(gray_img: &GrayImage, texture: f64) -> f64 {
    let lap = laplacian_filter(gray_img);
    let n = (lap.width() * lap.height()) as f64;
    if n < 2.0 {
        return 0.0;
    }

    let mean = lap.pixels().map(|p| p[0] as f64).sum::<f64>() / n;
    let variance = lap
        .pixels()
        .map(|p| {
            let v = p[0] as f64 - mean;
            v * v
        })
        .sum::<f64>()
        / (n - 1.0);

    let adjusted_var = variance * calculate_contrast(gray_img) * (1.0 - 0.5 * texture);

    ((adjusted_var - 50.0) / 950.0).clamp(0.0, 1.0)
}

fn calculate_noise(gray_img: &GrayImage, texture: f64) -> f64 {
    let denoised = median_filter(gray_img, 5, 5);
    let n = (gray_img.width() * gray_img.height()) as f64;

    let mean_diff = gray_img
        .pixels()
        .zip(denoised.pixels())
        .map(|(p, d)| (p[0] as f64 - d[0] as f64).abs())
        .sum::<f64>()
        / n;

    let raw_noise = (1.0 - ((mean_diff - 2.0) / 13.0)).clamp(0.0, 1.0);
    raw_noise * (1.0 - 0.5 * texture)
}

fn calculate_exposure(gray_img: &GrayImage) -> f64 {
    let n = (gray_img.width() * gray_img.height()) as f64;
    let clipped = gray_img
        .pixels()
        .filter(|p| p[0] < 40 || p[0] > 215)
        .count() as f64;

    (1.0 - clipped / n).clamp(0.0, 1.0)
}
