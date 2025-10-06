pub mod scan_all;

use color_eyre::Result;
use photos_core::{get_db_pool, get_media_dir, get_thumbnails_dir};
use scan_all::sync_files_to_db;
use std::time::Duration;
use tokio::{fs, time};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
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
                error!("Scanning failed: {}", e);
            }
        });
        // Wait for the next tick (24 hours)
    }
}

async fn run_scan() -> Result<()> {
    let pool = get_db_pool().await?;
    fs::create_dir_all(&get_thumbnails_dir()).await?;
    let media_dir = get_media_dir();

    info!("Scanning \"{}\" ...", &media_dir.display());
    sync_files_to_db(&media_dir, &pool).await?;
    info!("Scan complete");

    Ok(())
}
