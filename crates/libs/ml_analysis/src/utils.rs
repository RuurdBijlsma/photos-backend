use image::imageops::FilterType;
use std::io;
use std::path::Path;
use tokio::process::Command;
use tracing::warn;

pub async fn convert_media_file(
    input_path: &Path,
    output_path: &Path,
    image_out_size: u64,
) -> io::Result<()> {
    let input = input_path.to_owned();
    let output = output_path.to_owned();
    let image_result = tokio::task::spawn_blocking(move || {
        let img = image::open(&input).map_err(|e| io::Error::other(e.to_string()))?;
        let resized = img.resize(u32::MAX, image_out_size as u32, FilterType::Lanczos3);

        resized
            .save(&output)
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok::<(), io::Error>(())
    })
    .await;
    if matches!(image_result, Ok(Ok(()))) {
        return Ok(());
    }

    // 2. Fallback to ffmpeg
    warn!(
        "Failed to convert {} using `image` crate. Falling back to ffmpeg.",
        input_path.display()
    );
    let scale_filter = format!("scale=-2:{image_out_size}");
    let ffmpeg_output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(scale_filter)
        .arg("-y") // Overwrite output files without asking
        .arg(output_path)
        .output()
        .await?;

    if ffmpeg_output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(String::from_utf8_lossy(
            &ffmpeg_output.stderr,
        )))
    }
}
