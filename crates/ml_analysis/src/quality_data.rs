use crate::QualityData;
use color_eyre::eyre::Result;
use image::{imageops, GrayImage, DynamicImage};
use imageproc::filter::{laplacian_filter, median_filter};
use std::path::Path;

pub fn get_quality_data(image_path: &Path) -> Result<QualityData> {
    let img = image::ImageReader::open(image_path)?.decode()?;
    let img = resize_if_large(img, 1024);
    let gray_img = img.to_luma8();

    let blurriness = calculate_blurriness(&gray_img);
    let noisiness = calculate_noise(&gray_img);
    let exposure = calculate_exposure(&gray_img);

    let exposure_goodness = 1.0 - (exposure - 0.5).abs() * 2.0;

    let final_score = (blurriness * 0.5 + noisiness * 0.3 + exposure_goodness * 0.2)
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

fn calculate_blurriness(gray_img: &GrayImage) -> f64 {
    let laplacian = laplacian_filter(gray_img);
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let n = (laplacian.width() * laplacian.height()) as f64;

    if n < 2.0 {
        return 0.0;
    }

    for p in laplacian.pixels() {
        let v = p[0] as f64;
        sum += v;
        sum_sq += v * v;
    }

    let mean = sum / n;
    let variance = (sum_sq - n * mean * mean) / (n - 1.0);

    match variance {
        v if v <= 100.0 => 0.0,
        v if v >= 1000.0 => 1.0,
        v => (v - 100.0) / 900.0,
    }
}

fn calculate_noise(gray_img: &GrayImage) -> f64 {
    let denoised = median_filter(gray_img, 3, 3);
    let mut sum_diff = 0.0;
    let n = (gray_img.width() * gray_img.height()) as f64;

    for (p1, p2) in gray_img.pixels().zip(denoised.pixels()) {
        sum_diff += (p1[0] as f64 - p2[0] as f64).abs();
    }

    let mean_diff = sum_diff / n;

    match mean_diff {
        d if d >= 10.0 => 0.0,
        d if d <= 2.0 => 1.0,
        d => 1.0 - (d - 2.0) / 8.0,
    }
}

fn calculate_exposure(gray_img: &GrayImage) -> f64 {
    let sum: f64 = gray_img.pixels().map(|p| p[0] as f64).sum();
    sum / ((gray_img.width() * gray_img.height()) as f64 * 255.0)
}
