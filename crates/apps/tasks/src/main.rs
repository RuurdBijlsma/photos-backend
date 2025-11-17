use color_eyre::Result;
use tasks::task_runner::run_tasks;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use app_state::load_app_settings;
use common_services::database::get_db_pool;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;

    let settings = load_app_settings()?;
    let pool = get_db_pool(&settings.secrets.database_url).await?;
    run_tasks(pool, settings).await?;

    Ok(())
}
