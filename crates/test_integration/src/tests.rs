use crate::TestContext;
use color_eyre::eyre::Result;
use tracing::info;

pub async fn run_all(ctx: &TestContext) -> Result<()> {
    info!("--- Running Test: test_file_ingestion ---");
    test_auth(ctx).await?;
    test_file_ingestion(ctx).await?;
    info!("--- Test Passed: test_file_ingestion ---");

    // Add more test calls here
    // info!("--- Running Test: test_another_feature ---");
    // test_another_feature(ctx).await?;
    // info!("--- Test Passed: test_another_feature ---");

    Ok(())
}

async fn test_auth(ctx: &TestContext) -> Result<()> {
    Ok(())
}

async fn test_file_ingestion(_ctx: &TestContext) -> Result<()> {
    // ARRANGE
    // ACT
    // ASSERT

    Ok(())
}
