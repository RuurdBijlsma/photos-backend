use clap::Parser;
use color_eyre::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use worker::worker::create_worker;

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

    create_worker(Args::parse().analysis).await?;

    Ok(())
}
