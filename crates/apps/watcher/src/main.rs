mod handlers;
mod watcher;

use crate::watcher::start_watching;
use app_state::load_app_settings;
use color_eyre::Result;
use common_services::database::get_db_pool;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;

    let settings = load_app_settings()?;
    let pool = get_db_pool(&settings.secrets.database_url, true).await?;
    start_watching(pool, settings).await?;

    Ok(())
}
