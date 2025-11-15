use super::error::S2sClientError;
use crate::{
    api::album::interfaces::AlbumShareClaims, database::album::album::AlbumSummary,
    get_settings::settings,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest::Client;
use url::Url;

/// Parses an invite token to extract the claims, including the remote server URL.
pub fn extract_token_claims(token: &str) -> Result<AlbumShareClaims, S2sClientError> {
    decode::<AlbumShareClaims>(
        token,
        &DecodingKey::from_secret(settings().auth.jwt_secret.as_ref()),
        &Validation::default(),
    )
        .map(|t| t.claims)
        .map_err(Into::into)
}

#[derive(Clone)]
pub struct S2sClient {
    http_client: Client,
}

impl S2sClient {
    pub fn new(http_client: Client) -> Self {
        Self { http_client }
    }

    /// Fetches the summary of a shared album from a remote server using an invite token.
    pub async fn get_album_invite_summary(
        &self,
        token: &str,
    ) -> Result<AlbumSummary, S2sClientError> {
        let claims = extract_token_claims(token)?;
        let mut remote_url: Url = claims.iss.parse()?;
        remote_url.set_path("/s2s/albums/invite-summary");

        let response = self
            .http_client
            .get(remote_url.clone())
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(S2sClientError::RemoteServerError(format!(
                "Remote server {remote_url} returned an error: {error_text}"
            )));
        }

        let summary: AlbumSummary = response.json().await?;
        Ok(summary)
    }
}