mod handlers;
mod watcher;

use color_eyre::Result;
use common_services::alert;
use common_services::database::get_db_pool;
use common_services::get_settings::media_dir;
use tracing::Level;
use tracing::warn;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;

    let pool = get_db_pool().await?;
    let media_path = media_dir();

    if let Err(e) = watcher::run(media_path, pool).await {
        alert!("Watcher failed with an error: {}", e);
    }

    Ok(())
}
