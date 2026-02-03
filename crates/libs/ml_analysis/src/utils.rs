use crate::PyInterop;
use color_eyre::eyre::ContextCompat;
use common_types::variant::Variant;
use pyo3::Python;
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
pub fn get_color_theme(
    color: &str,
    variant: &Variant,
    contrast_level: f32,
) -> color_eyre::Result<Value> {
    Python::attach(|py| {
        let interop = PyInterop::new(py).ok()?;
        Some(interop.get_theme_from_color(color, variant, contrast_level))
    })
    .context("Python attach failed for theme color generation")?
}
