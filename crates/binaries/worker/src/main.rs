use clap::Parser;
use color_eyre::Result;
use common_services::database::get_db_pool;
use common_services::utils::nice_id;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use worker::context::WorkerContext;
use worker::worker::run_worker_loop;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(long, default_value_t = false, short, action)]
    analysis: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;
    let args = Args::parse();

    let worker_id = nice_id(8);
    info!("[Worker ID: {}] Starting.", worker_id);

    let pool = get_db_pool().await?;
    let context = WorkerContext::new(pool, worker_id.clone(), args.analysis).await?;

    run_worker_loop(&context).await?;

    Ok(())
}
