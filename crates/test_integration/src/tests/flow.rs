use crate::test_context::TestContext;
use color_eyre::Result;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use common_services::api::auth::interfaces::CreateUser;
use common_services::database::app_user::{User, UserRole};

async fn run_test<'a, F, Fut>(name: &str, ctx: &'a TestContext, f: F) -> Result<()>
where
    F: Fn(&'a TestContext) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    info!("Running test '{}'", name);
    let result = f(ctx).await;
    if result.is_ok() { info!("✅ {} Passed.", name) } else { warn!("⚠️ {} Failed.", name) }
    if let Err(e) = result {
        println!("{e}");
    }

    Ok(())
}

#[tokio::test]
async fn integration_test() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");
    color_eyre::install().expect("Failed to install color_eyre");

    let context = TestContext::new().await?;

    // Pass async test functions using closures
    run_test("test_health_endpoint", &context, |ctx| async move {
        test_health_endpoint(ctx).await
    }).await?;

    run_test("test_auth", &context, |ctx| async move {
        test_auth(ctx).await
    }).await?;

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

async fn test_auth(context: &TestContext) -> Result<()> {
    // ARRANGE
    let client = &context.http_client;
    let url = format!("{}/auth/register", &context.settings.api.public_url);
    let name = "ruurd".to_owned();
    let email = "ruurd@bijlsma.dev".to_owned();
    let password = "hi-there".to_owned();

    // ACT
    let response = client
        .post(url)
        .json(&CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: password.clone(),
        })
        .send()
        .await?;
    let status = response.status();
    let user: User = response.json().await?;

    // ASSERT
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(user.name, name);
    assert_eq!(user.email, email);
    assert_eq!(user.media_folder, None);
    assert_eq!(user.role, UserRole::Admin);

    Ok(())
}
