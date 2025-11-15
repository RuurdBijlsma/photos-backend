mod watcher;

use crate::watcher::start_watching;
use color_eyre::Result;
use common_services::database::get_db_pool;
use common_services::get_settings::media_dir;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;

    let pool = get_db_pool().await?;

    info!("Start watching for file changes...");
    start_watching(media_dir(), &pool);

    Ok(())
}
