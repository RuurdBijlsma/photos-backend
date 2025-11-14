use ml_analysis::get_quality_data;
use std::path::Path;
use std::time::Instant;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let images = vec![
        Path::new("media_dir/rutenl/tree.jpg"),
        Path::new("media_dir/rutenl/sunset.jpg"),
        Path::new("media_dir/rutenl/pics/PICT0017.JPG"),
    ];

    for image in images {
        let now = Instant::now();
        let quality_data = get_quality_data(image)?;
        println!(
            "{} quality: {:?}",
            image.file_name().unwrap().to_string_lossy(),
            quality_data
        );
        println!("\tget_quality_data {:?}", now.elapsed());
    }

    Ok(())
}
