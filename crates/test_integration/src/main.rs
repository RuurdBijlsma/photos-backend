extern crate core;

use crate::test_context::TestContext;
use color_eyre::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod test_context;
mod tests;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    color_eyre::install()?;

    let ctx = TestContext::new().await?;

    // Run our tests
    tests::run_all(&ctx).await?;

    Ok(())
}