use crate::helpers::test_context::test_context::TestContext;
use color_eyre::Result;
use common_services::api::auth::interfaces::CreateUser;
use common_services::database::app_user::{User, UserRole};

pub async fn test_health_endpoint(context: &TestContext) -> Result<()> {
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

pub async fn test_auth(context: &TestContext) -> Result<()> {
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
