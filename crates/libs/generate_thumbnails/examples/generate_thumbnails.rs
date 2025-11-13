use common_services::utils::get_thumb_options;
use generate_thumbnails::generate_thumbnails;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let file = Path::new("media_dir/rutenl/vids/jellyfish.mp4");
    let out_folder = Path::new("test_out");
    fs::create_dir_all(out_folder)?;
    generate_thumbnails(file, out_folder, &get_thumb_options(), Some(5)).await?;

    Ok(())
}
