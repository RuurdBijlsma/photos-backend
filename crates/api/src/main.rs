use tracing::info;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    info!("Hello, world!");

    Ok(())
}
