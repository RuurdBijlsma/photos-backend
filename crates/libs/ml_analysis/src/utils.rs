use crate::VisualAnalyzer;
use common_services::settings::settings;
use serde_json::Value;
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
pub fn get_color_theme(color: &str) -> color_eyre::Result<Value> {
    let visual_analyzer = VisualAnalyzer::new()?;
    let variant = &settings().analyzer.theme_generation.variant;
    let contrast_level = settings().analyzer.theme_generation.contrast_level;
    let theme = visual_analyzer.theme_from_color(color, variant, contrast_level as f32)?;
    Ok(theme)
}
