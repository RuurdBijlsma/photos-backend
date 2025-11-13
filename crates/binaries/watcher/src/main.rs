mod watcher;

use crate::watcher::start_watching;
use color_eyre::Result;
use common_services::settings::media_dir;
use common_services::utils::get_db_pool;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let pool = get_db_pool().await?;

    info!("Start watching for file changes...");
    start_watching(media_dir(), &pool);

    Ok(())
}
