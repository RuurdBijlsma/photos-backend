use crate::TestContext;
use color_eyre::eyre::Result;
use sqlx::Row;
use tokio::time::{Duration};
use tracing::info;

pub async fn run_all(ctx: &TestContext) -> Result<()> {
    info!("--- Running Test: test_file_ingestion ---");
    test_file_ingestion(ctx).await?;
    info!("--- Test Passed: test_file_ingestion ---");

    // Add more test calls here
    // info!("--- Running Test: test_another_feature ---");
    // test_another_feature(ctx).await?;
    // info!("--- Test Passed: test_another_feature ---");

    Ok(())
}

async fn test_file_ingestion(ctx: &TestContext) -> Result<()> {
    // ARRANGE

    // ACT
    // ASSERT

    Ok(())
}