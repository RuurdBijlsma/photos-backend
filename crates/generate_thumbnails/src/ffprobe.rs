use color_eyre::eyre::{Context, bail};
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Deserialize)]
struct FfprobeOutput {
    format: FormatInfo,
}

#[derive(Deserialize)]
struct FormatInfo {
    duration: String,
}

/// Executes ffprobe command and returns its stdout as a String.
pub async fn run_ffprobe<S: AsRef<OsStr>>(args: &[S]) -> color_eyre::Result<String> {
    let output = Command::new("ffprobe")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("failed to run ffprobe")?;

    if output.status.success() {
        // If the command was successful, convert the stdout bytes to a String
        String::from_utf8(output.stdout).context("ffprobe output was not valid UTF-8")
    } else {
        // If the command failed, create an error from the standard error output
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffprobe failed: {}", stderr.trim());
    }
}

/// Gets the duration of a video file in seconds.
pub async fn get_video_duration(video_path: &Path) -> color_eyre::Result<f64> {
    let Some(video_path_str) = video_path.as_os_str().to_str() else {
        bail!("ffprobe video path is not valid UTF-8");
    };

    let args = &[
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_format",
        video_path_str,
    ];

    // Get the raw JSON output from ffprobe
    let ffprobe_json = run_ffprobe(args).await?;

    // Parse the JSON string into our structs
    let ffprobe_data: FfprobeOutput =
        serde_json::from_str(&ffprobe_json).context("Failed to parse ffprobe JSON output")?;

    // Parse the duration string from the struct into a f64
    let duration: f64 = ffprobe_data
        .format
        .duration
        .parse()
        .context("Failed to parse duration string into a number")?;

    Ok(duration)
}
