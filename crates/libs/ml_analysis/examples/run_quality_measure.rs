use ml_analysis::get_quality_measurement;
use std::path::{Path, PathBuf};
use std::time::Instant;
use image::{imageops, DynamicImage};
use walkdir::WalkDir;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let target_dir = Path::new("C:/Users/Ruurd/Pictures/small_media_dir");
    let mut results = Vec::new();

    println!("Scanning directory: {}", target_dir.display());
    let scan_start = Instant::now();

    // Recursively walk through the directory
    for entry in WalkDir::new(target_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
                    let now = Instant::now();

                    // Gracefully handle individual image read errors without crashing the scan
                    match get_quality_measurement(path) {
                        Ok(quality_data) => {
                            println!(
                                "Analyzed {} in {:?}",
                                path.file_name().unwrap().to_string_lossy(),
                                now.elapsed()
                            );
                            results.push((path.to_path_buf(), quality_data));
                        }
                        Err(err) => {
                            eprintln!("Error processing {}: {:?}", path.display(), err);
                        }
                    }
                }
            }
        }
    }

    println!(
        "\nFinished scanning {} images in {:?}",
        results.len(),
        scan_start.elapsed()
    );

    // Sort by weighted_score in ascending order (worst quality first)
    results.sort_by(|a, b| {
        a.1.weighted_score
            .partial_cmp(&b.1.weighted_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Print sorted results
    println!("\n=====================================================================================================");
    println!("                           QUALITY ANALYSIS RESULTS (Worst to Best)                                  ");
    println!("=====================================================================================================");
    println!(
        "{:<6} | {:<14} | {:<10} | {:<9} | {:<8} | {}",
        "Score", "Accidentalness", "Blurriness", "Noisiness", "Exposure", "File Path"
    );
    println!("-----------------------------------------------------------------------------------------------------");

    for (path, q) in &results {
        println!(
            "{:<6.1} | {:<14.3} | {:<10.3} | {:<9.3} | {:<8.3} | {}",
            q.weighted_score,
            q.accidentalness,
            q.blurriness,
            q.noisiness,
            q.exposure,
            path.display()
        );
    }
    println!("=====================================================================================================");

    Ok(())
}

fn resize_if_large(img: DynamicImage, max_dim: u32) -> DynamicImage {
    if img.width() > max_dim || img.height() > max_dim {
        img.resize(max_dim, max_dim, imageops::FilterType::Lanczos3)
    } else {
        img
    }
}