use crate::{ColorData, ColorHistogram, PyInterop, RGBChannels, Variant};
use image::Rgb;
use palette::{FromColor, Hsv, Srgb};
use std::path::Path;

fn average_hue_from_sums(x_sum: f32, y_sum: f32) -> f32 {
    let mut avg = y_sum.atan2(x_sum).to_degrees();
    if avg < 0.0 {
        avg += 360.0;
    }
    avg
}

pub fn get_color_data(
    py_interop: &PyInterop,
    file: &Path,
    theme_variant: &Variant,
    theme_contrast_level: f32,
) -> color_eyre::Result<ColorData> {
    let rgb_image = image::open(file)?.to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let pixel_count = (width * height) as f32;

    let mut hist_r = [0i32; 256];
    let mut hist_g = [0i32; 256];
    let mut hist_b = [0i32; 256];

    let mut x_sum = 0.0;
    let mut y_sum = 0.0;
    let mut sat_sum = 0.0;
    let mut val_sum = 0.0;

    for Rgb([r, g, b]) in rgb_image.pixels() {
        hist_r[*r as usize] += 1;
        hist_g[*g as usize] += 1;
        hist_b[*b as usize] += 1;

        let hsv = Hsv::from_color(Srgb::new(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0).into_linear());
        let rad = hsv.hue.into_radians();
        x_sum += rad.cos();
        y_sum += rad.sin();
        sat_sum += hsv.saturation;
        val_sum += hsv.value;
    }

    let average_hue = average_hue_from_sums(x_sum, y_sum);
    let average_saturation = sat_sum / pixel_count * 100.0;
    let average_lightness = val_sum / pixel_count * 100.0;

    let prominent_colors = py_interop.get_image_prominent_colors(file)?;
    let themes = prominent_colors
        .iter()
        .map(|c| py_interop.get_theme_from_color(c, theme_variant, theme_contrast_level))
        .collect::<Result<Vec<_>, _>>()?;

    let histogram = ColorHistogram {
        bins: 256,
        channels: RGBChannels {
            red: hist_r.to_vec(),
            green: hist_g.to_vec(),
            blue: hist_b.to_vec(),
        },
    };

    Ok(ColorData {
        themes,
        prominent_colors,
        average_hue,
        average_saturation,
        average_lightness,
        histogram,
    })
}
