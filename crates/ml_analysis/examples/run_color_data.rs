use ml_analysis::{PyInterop, get_color_data};
use pyo3::Python;
use std::path::Path;
use std::time::Instant;
use common_photos::Variant;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let images = vec![
        Path::new("media_dir/rutenl/tree.jpg"),
        Path::new("media_dir/rutenl/sunset.jpg"),
        Path::new("media_dir/rutenl/pics/PICT0017.JPG"),
    ];

    Python::attach(|py| {
        let py_interop = PyInterop::new(py).unwrap();

        for image in images {
            let now = Instant::now();
            let color_data = get_color_data(&py_interop, image, &Variant::Vibrant, 3.).unwrap();
            println!(
                "{} color: {:?} {:?} {:?} {:?}",
                image.file_name().unwrap().to_string_lossy().to_string(),
                color_data.prominent_colors,
                color_data.average_hue,
                color_data.average_saturation,
                color_data.average_lightness,
            );
            println!("\tget_color_data {:?}", now.elapsed());
        }
    });

    Ok(())
}
