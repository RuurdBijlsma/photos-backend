mod watcher;

use crate::watcher::start_watching;
use color_eyre::Result;
use photos_core::get_db_pool;
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let pool = get_db_pool().await?;

    let media_dir = Path::new("assets");
    let thumbs_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbs_dir).await?;

    println!("Start watching for file changes...");
    start_watching(media_dir, &pool)?;

    Ok(())
}
