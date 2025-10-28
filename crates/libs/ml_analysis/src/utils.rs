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
