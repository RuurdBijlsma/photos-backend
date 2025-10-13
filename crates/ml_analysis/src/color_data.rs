use crate::{ColorData, ColorHistogram, RGBChannels};
use image::{DynamicImage, Rgb};
use palette::{FromColor, Hsv, Srgb};
use serde_json::json;

/// placeholder for theme generation logic
fn generate_themes(
    prominent_colors: &[String],
    variant: &str,
    contrast: f32,
) -> Vec<serde_json::Value> {
    // todo user mateiral cololr utlis
    prominent_colors
        .iter()
        .map(|color| {
            json!({
                "base_color": color,
                "variant": variant,
                "contrast": contrast
            })
        })
        .collect()
}

fn average_hue(hues: &[f32]) -> f32 {
    let n = hues.len() as f32;
    if n == 0.0 {
        return 0.0;
    }
    let (x_sum, y_sum): (f32, f32) = hues
        .iter()
        .map(|h| h.to_radians())
        .map(|r| (r.cos(), r.sin()))
        .fold((0.0, 0.0), |a, (x, y)| (a.0 + x, a.1 + y));
    let avg_x = x_sum / n;
    let avg_y = y_sum / n;
    let mut avg_hue = avg_y.atan2(avg_x).to_degrees();
    if avg_hue < 0.0 {
        avg_hue += 360.0;
    }
    avg_hue
}

pub fn analyze_colors(
    image: &DynamicImage,
    theme_color_variant: &str,
    theme_contrast_level: f32,
) -> ColorData {
    let rgb_image = image.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    let mut hues = Vec::with_capacity((width * height) as usize);
    let mut sats = Vec::with_capacity((width * height) as usize);
    let mut vals = Vec::with_capacity((width * height) as usize);

    let mut hist_r = vec![0i32; 256];
    let mut hist_g = vec![0i32; 256];
    let mut hist_b = vec![0i32; 256];

    for pixel in rgb_image.pixels() {
        let Rgb([r, g, b]) = *pixel;

        hist_r[r as usize] += 1;
        hist_g[g as usize] += 1;
        hist_b[b as usize] += 1;

        let rgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let hsv = Hsv::from_color(rgb.into_linear());
        hues.push(hsv.hue.into_degrees());
        sats.push(hsv.saturation);
        vals.push(hsv.value);
    }

    let average_hue = average_hue(&hues);
    let average_saturation = sats.iter().sum::<f32>() / sats.len() as f32 * 100.0;
    let average_lightness = vals.iter().sum::<f32>() / vals.len() as f32 * 100.0;

    let prominent_colors = vec![
        "#ff0000".to_string(),
        "#00ff00".to_string(),
        "#0000ff".to_string(),
    ];

    let themes = generate_themes(&prominent_colors, theme_color_variant, theme_contrast_level);

    let histogram = ColorHistogram {
        bins: 256,
        channels: RGBChannels {
            red: hist_r,
            green: hist_g,
            blue: hist_b,
        },
    };

    ColorData {
        themes,
        prominent_colors,
        average_hue,
        average_saturation,
        average_lightness,
        histogram,
    }
}
