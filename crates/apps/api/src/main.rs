use api::serve;
use app_state::load_app_settings;
use color_eyre::Result;
use common_services::database::get_db_pool;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    color_eyre::install()?;

    let settings = load_app_settings()?;
    let pool = get_db_pool(&settings.secrets.database_url, true).await?;
    serve(pool, settings).await?;

    Ok(())
}
