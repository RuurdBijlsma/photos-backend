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
    let name = "Ruurd".to_owned();
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
    
    let created_user = sqlx::query_as!(
        User,
        r#"SELECT id, email, name, media_folder, role as "role: UserRole", created_at, updated_at
           FROM app_user"#
    )
    .fetch_all(&context.pool)
    .await?;
    assert_eq!(created_user.len(), 1);
    let created_user = &created_user[0];
    assert_eq!(created_user.name, name);
    assert_eq!(created_user.email, email);
    assert_eq!(created_user.media_folder, None);
    assert_eq!(created_user.role, UserRole::Admin);

    Ok(())
}

pub async fn test_second_register_attempt(context: &TestContext) -> Result<()> {
    // ARRANGE
    let client = &context.http_client;
    let url = format!("{}/auth/register", &context.settings.api.public_url);
    let name = "karel".to_owned();
    let email = "karel@bijlsma.dev".to_owned();
    let password = "my-password".to_owned();

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

    // ASSERT
    assert_eq!(status, reqwest::StatusCode::FORBIDDEN);

    Ok(())
}
