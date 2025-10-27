use crate::ffmpeg::run_ffmpeg;
use crate::thumbnails::utils::map_still;
use color_eyre::eyre::ContextCompat;
use std::path::Path;
use tokio::fs;
use common_photos::ThumbOptions;

pub async fn generate_ffmpeg_photo_thumbnails(
    input: &Path,
    output_dir: &Path,
    config: &ThumbOptions,
) -> color_eyre::Result<()> {
    let heights = &config.heights;
    if config.heights.is_empty() {
        return Ok(());
    }
    let ext = &config.thumbnail_extension;

    fs::create_dir_all(output_dir).await?;
    let input_str = input.to_str().context("invalid input path")?;

    let split_labels: Vec<String> = (0..heights.len()).map(|i| format!("[v{i}]")).collect();
    let mut filter_parts = vec![format!(
        "[0:v]split={}{}",
        heights.len(),
        split_labels.join(""),
    )];

    let mut args = vec!["-y".into(), "-i".into(), input_str.into()];
    let mut map_args = Vec::new();

    for (i, &h) in heights.iter().enumerate() {
        let out_label = format!("[out{i}]");
        filter_parts.push(format!("[v{i}]scale=-1:{h}{out_label}"));
        let out = output_dir.join(format!("{h}p.{ext}"));
        map_args.extend(map_still(&out_label, &out));
    }

    args.push("-filter_complex".into());
    args.push(filter_parts.join(";"));
    args.extend(map_args);

    run_ffmpeg(&args).await
}
