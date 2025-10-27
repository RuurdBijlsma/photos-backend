use color_eyre::eyre::{Context, bail};
use std::ffi::OsStr;
use std::process::Stdio;
use tokio::process::Command;

pub async fn run_ffmpeg<S: AsRef<OsStr>>(args: &[S]) -> color_eyre::Result<()> {
    let output = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("failed to run ffmpeg")?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffmpeg failed: {}", stderr.trim());
    }
}
