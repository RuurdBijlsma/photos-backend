use crate::ffmpeg::{get_video_duration, FfmpegCommand};
use color_eyre::eyre::Result;
use std::path::Path;
use tokio::fs;
use app_state::ThumbnailSettings;

pub async fn generate_video_thumbnails(
    input: &Path,
    output_dir: &Path,
    config: &ThumbnailSettings,
) -> Result<()> {
    fs::create_dir_all(output_dir).await?;

    let duration = get_video_duration(input).await?;
    let mut cmd = FfmpegCommand::new(input);

    // 1. Generate multi-size stills from a fixed timestamp.
    generate_fixed_time_stills(&mut cmd, output_dir, config);

    // 2. Generate stills from various time percentages.
    generate_percentage_stills(&mut cmd, output_dir, config, duration);

    // 3. Generate transcoded video previews.
    generate_video_transcodes(&mut cmd, output_dir, config);

    cmd.run().await
}

fn generate_fixed_time_stills(
    cmd: &mut FfmpegCommand,
    output_dir: &Path,
    config: &ThumbnailSettings,
) {
    if config.heights.is_empty() {
        return;
    }
    let input_stream = cmd.add_input_at_time(config.video_options.thumb_time);
    let split_streams = cmd.add_split(&input_stream, config.heights.len());

    for (i, &h) in config.heights.iter().enumerate() {
        let scaled_stream = cmd.add_scale(&split_streams[i], -1, h as i32);
        let out_path = output_dir.join(format!("{h}p.{}", config.thumbnail_extension));
        cmd.map_still_output(&scaled_stream, &out_path);
    }
}

fn generate_percentage_stills(
    cmd: &mut FfmpegCommand,
    output_dir: &Path,
    config: &ThumbnailSettings,
    duration: f64,
) {
    if config.video_options.percentages.is_empty() {
        return;
    }
    let target_h = config.video_options.height;
    let thumb_ext = &config.thumbnail_extension;

    for &pct in &config.video_options.percentages {
        let ts = (pct as f64) / 100.0 * duration;
        let input_stream = cmd.add_input_at_time(ts);
        let scaled_stream = cmd.add_scale(&input_stream, -1, target_h as i32);
        let out_path = output_dir.join(format!("{pct}_percent.{thumb_ext}"));
        cmd.map_still_output(&scaled_stream, &out_path);
    }
}

fn generate_video_transcodes(
    cmd: &mut FfmpegCommand,
    output_dir: &Path,
    config: &ThumbnailSettings,
) {
    if config.video_options.transcode_outputs.is_empty() {
        return;
    }
    // This input corresponds to the main video file, added when the command was created.
    let main_video_stream = "[0:v:0]";
    let main_audio_stream = "[0:a:0?]";
    let num_outputs = config.video_options.transcode_outputs.len();

    let v_streams = cmd.add_split(main_video_stream, num_outputs);
    let a_streams = cmd.add_asplit(main_audio_stream, num_outputs);

    for (i, hq_config) in config.video_options.transcode_outputs.iter().enumerate() {
        let h = hq_config.height;
        let scaled_v_stream = cmd.add_scale(&v_streams[i], -2, h as i32);
        let out_path = output_dir.join(format!("{h}p.{}", config.video_options.extension));

        cmd.map_video_output(
            &scaled_v_stream,
            &a_streams[i],
            hq_config.quality,
            &out_path,
        );
    }
}
