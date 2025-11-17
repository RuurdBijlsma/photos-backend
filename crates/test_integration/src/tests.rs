use crate::TestContext;
use color_eyre::eyre::Result;
use tracing::{info, warn};

pub async fn run_all(ctx: &TestContext) -> Result<()> {
    info!("--- Running Test: test_health_check ---");
    let result = test_health_check(ctx).await;
    match result {
        Ok(_) => info!("--- ✅ Test Passed: test_health_check ---"),
        Err(e) => warn!("--- ⚠️ Test Failed: test_health_check --- {:?}", e),
    }

    info!("--- Running Test: test_auth ---");
    let result = test_auth(ctx).await;
    match result {
        Ok(_) => info!("--- ✅ Test Passed: test_auth ---"),
        Err(e) => warn!("--- ⚠️ Test Failed: test_auth --- {:?}", e),
    }

    info!("--- Running Test: test_file_ingestion ---");
    let result = test_file_ingestion(ctx).await;
    match result {
        Ok(_) => info!("--- ✅ Test Passed: test_file_ingestion ---"),
        Err(e) => warn!("--- ⚠️ Test Failed: test_file_ingestion --- {:?}", e),
    }

    // Add more test calls here
    // info!("--- Running Test: test_another_feature ---");
    // test_another_feature(ctx).await?;
    // info!("--- Test Passed: test_another_feature ---");

    Ok(())
}

async fn test_health_check(ctx: &TestContext) -> Result<()> {
    // ARRANGE
    let client = &ctx.http_client;
    let url = format!("{}/health", &ctx.settings.api.public_url);

    // ACT
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    // ASSERT
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body, "OK");

    Ok(())
}

async fn test_auth(_ctx: &TestContext) -> Result<()> {
    Ok(())
}

async fn test_file_ingestion(_ctx: &TestContext) -> Result<()> {
    // ARRANGE
    // ACT
    // ASSERT

    Ok(())
}
