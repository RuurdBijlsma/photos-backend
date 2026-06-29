use std::path::Path;
use color_eyre::Result;
use panorama_tiler::{tile_panorama_with_guessed_angles, DownscalingMethod, OutputConfig, OutputFormat};

pub fn generate_pano_thumbs(input_file: &Path, output_folder: &Path) -> Result<()> {


    let output_config = OutputConfig {
        format: OutputFormat::Webp,
        quality: 85,
        downscaling_method: DownscalingMethod::Direct,
        ..Default::default()
    };

    tile_panorama_with_guessed_angles(input_file, output_folder, Some(output_config))?;

    Ok(())
}
