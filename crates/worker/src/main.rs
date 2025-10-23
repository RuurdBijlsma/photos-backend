#![allow(clippy::cognitive_complexity)]

use clap::Parser;
use color_eyre::Result;
use common_photos::{get_db_pool, nice_id};
use tracing::info;
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
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let args = Args::parse();

    let worker_id = nice_id(8);
    info!("[Worker ID: {}] Starting.", worker_id);

    let pool = get_db_pool().await?;
    let context = WorkerContext::new(pool, worker_id.to_string(), args.analysis).await?;

    run_worker_loop(&context).await?;

    Ok(())
}
