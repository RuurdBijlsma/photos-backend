use ml_analysis::VisualAnalyzer;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let now = Instant::now();
    let analyzer = VisualAnalyzer::new()?;
    println!("VisualAnalyzer::new {:?}\n", now.elapsed());

    let images = vec![
        Path::new("media_dir/rutenl/tree.jpg"),
        Path::new("media_dir/rutenl/road.jpg"),
        Path::new("media_dir/rutenl/cat-pet.jpg"),
        Path::new("media_dir/rutenl/sunset.jpg"),
        Path::new("media_dir/rutenl/pics/PICT0017.JPG"),
        Path::new("media_dir/rutenl/pizza.jpg"),
    ];

    for image in images {
        let image_filename = image.file_name().unwrap().to_string_lossy().to_string();
        println!("analyze image {image_filename}");
        let now = Instant::now();
        let analysis = analyzer.analyze_image(image)?;
        let filename = format!("{image_filename}-analysis.json");
        let file = File::create(Path::new(&filename))?;
        serde_json::to_writer_pretty(file, &analysis)?;
        println!("{:?}\n", now.elapsed());
    }

    Ok(())
}
