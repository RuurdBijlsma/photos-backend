use crate::test_context::TestContext;
use color_eyre::eyre::Result;
use tokio::sync::OnceCell; // Replaced std::sync::LazyLock
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

// Use OnceCell for async-aware one-time initialization.
static CTX: OnceCell<TestContext> = OnceCell::const_new();

// This function will initialize the TestContext once and only once across all tests.
// Subsequent calls will return the already initialized context immediately.
async fn initialize_context() -> &'static TestContext {
    CTX.get_or_init(|| async {
        println!("--- Setting up shared TestContext for integration tests ---");
        TestContext::new()
            .await
            .expect("Failed to create TestContext")
    })
        .await
}

#[tokio::test]
async fn test_health_check() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");
    color_eyre::install().expect("Failed to install color_eyre");
    // ARRANGE
    // Get the shared context. This will initialize it on the first run.
    let context = initialize_context().await;
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

#[tokio::test]
async fn test_auth() -> Result<()> {
    // ARRANGE
    let _context = initialize_context().await;
    // ACT
    // ASSERT

    Ok(())
}

#[tokio::test]
async fn test_file_ingestion() -> Result<()> {
    // ARRANGE
    let _context = initialize_context().await;
    // ACT
    // ASSERT

    Ok(())
}