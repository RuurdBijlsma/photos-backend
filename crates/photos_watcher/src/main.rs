mod watcher;

use crate::watcher::start_watching;
use color_eyre::Result;
use photos_core::{get_db_pool, get_media_dir, get_thumbnails_dir};
use tokio::fs;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let pool = get_db_pool().await?;
    let thumbnails_dir = get_thumbnails_dir();
    let media_dir = get_media_dir();
    fs::create_dir_all(&thumbnails_dir).await?;

    info!("Start watching for file changes...");
    start_watching(&media_dir, &pool)?;

    Ok(())
}
