use crate::test_constants::{EMAIL, PASSWORD};
use color_eyre::Result;
use common_services::api::auth::interfaces::{LoginUser, Tokens};
use crate::runner::context::test_context::TestContext;

pub async fn login(context: &TestContext) -> Result<String> {
    let url = format!("{}/auth/login", &context.settings.api.public_url);
    let response = context
        .http_client
        .post(url)
        .json(&LoginUser {
            email: EMAIL.to_owned(),
            password: PASSWORD.to_owned(),
        })
        .send()
        .await?;
    let tokens: Tokens = response.json().await?;

    Ok(tokens.access_token)
}
