pub mod scan_all;

use color_eyre::Result;
use photos_core::{get_db_pool, get_thumbnail_options};
use scan_all::scan_all_files;
use std::path::Path;
use std::time::Duration;
use tokio::{fs, time};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let twenty_four_hours = Duration::from_secs(24 * 60 * 60);
    let mut interval = time::interval(twenty_four_hours);

    loop {
        // The first tick of `interval` happens immediately.
        interval.tick().await;

        tokio::spawn(async {
            let result = run_scan().await;
            if let Err(e) = result {
                // todo alert here
                eprintln!("Scanning failed: {}", e);
            }
        });
        // Wait for the next tick (24 hours)
    }
}

async fn run_scan() -> Result<()> {
    let pool = get_db_pool().await?;

    let media_dir = Path::new("assets");
    let thumbs_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbs_dir).await?;
    let config = get_thumbnail_options(thumbs_dir);

    println!("Scanning \"{}\" ...", media_dir.display());
    scan_all_files(media_dir, &config, &pool).await?;
    println!("Scan complete");

    Ok(())
}
