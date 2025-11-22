use crate::test_context::TestContext;
use color_eyre::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::test]
async fn test_main_flow() -> Result<()> {
    // SETUP TEST CONTEXT
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    color_eyre::install().expect("Failed to install color_eyre");
    let context = TestContext::new()
        .await
        .expect("Failed to create TestContext");

    test_health_endpoint(&context).await?;

    Ok(())
}

async fn test_health_endpoint(context: &TestContext) -> Result<()> {
    // ARRANGE
    let client = &context.http_client;
    let url = format!("{}/health", &context.settings.api.public_url);

    // ACT
    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    // ASSERT
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body, "OK");

    Ok(())
}
