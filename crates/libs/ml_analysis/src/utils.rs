use material_color_utils::dynamic::variant::Variant;
use material_color_utils::utils::color_utils::Argb;
use material_color_utils::{theme_from_color, MaterializedTheme};
use std::io;
use std::path::Path;
use tokio::process::Command;

/// Use ffmpeg to convert a photo or video.
pub async fn convert_media_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg(output_path)
        .arg("-y")
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(String::from_utf8_lossy(&output.stderr)))
    }
}

/// Generate material color theme from a color.
pub fn get_color_theme(
    color: &str,
    variant: Variant,
    contrast_level: f64,
) -> color_eyre::Result<MaterializedTheme> {
    let theme = theme_from_color(Argb::from_hex(color)?)
        .variant(variant)
        .contrast_level(contrast_level)
        .call();
    Ok(theme)
}
