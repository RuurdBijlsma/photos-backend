use common_types::ml_analysis::{MLColorData, MLColorHistogram, MLRGBChannels};
use image::Rgb;
use material_color_utils::dynamic::variant::Variant;
use material_color_utils::theme_from_color;
use palette::{FromColor, Hsv, Srgb};
use std::path::Path;
use material_color_utils::utils::color_utils::Argb;

fn average_hue_from_sums(x_sum: f32, y_sum: f32) -> f32 {
    let mut avg = y_sum.atan2(x_sum).to_degrees();
    if avg < 0.0 {
        avg += 360.0;
    }
    avg
}

/// Analyzes an image file to calculate its color properties, including prominent colors, color themes, average color values, and a histogram.
///
/// # Errors
///
/// This function will return an error if the image cannot be opened/decoded or if the Python interoperability calls fail.
pub fn get_color_data(
    file: &Path,
    theme_variant: &Variant,
    theme_contrast_level: f64,
) -> color_eyre::Result<MLColorData> {
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

        let hsv = Hsv::from_color(
            Srgb::new(
                f32::from(*r) / 255.0,
                f32::from(*g) / 255.0,
                f32::from(*b) / 255.0,
            )
            .into_linear(),
        );
        let rad = hsv.hue.into_radians();
        x_sum += rad.cos();
        y_sum += rad.sin();
        sat_sum += hsv.saturation;
        val_sum += hsv.value;
    }

    let average_hue = average_hue_from_sums(x_sum, y_sum);
    let average_saturation = sat_sum / pixel_count * 100.0;
    let average_lightness = val_sum / pixel_count * 100.0;

    let img = image::open(file)?;
    let prominent_colors = material_color_utils::extract_image_colors(&img).call();
    let themes = prominent_colors
        .iter()
        .map(|c| {
            theme_from_color(*c)
                .variant(*theme_variant)
                .contrast_level(theme_contrast_level)
                .call()
        })
        .collect();

    let histogram = MLColorHistogram {
        bins: 256,
        channels: MLRGBChannels {
            red: hist_r.to_vec(),
            green: hist_g.to_vec(),
            blue: hist_b.to_vec(),
        },
    };

    Ok(MLColorData {
        themes,
        prominent_colors: prominent_colors.iter().map(Argb::to_hex).collect(),
        average_hue,
        average_saturation,
        average_lightness,
        histogram,
    })
}
