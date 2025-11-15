use crate::alert;
use crate::api::album::interfaces::AlbumShareClaims;
use crate::database::album::album::AlbumSummary;
use crate::get_settings::settings;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use futures_util::StreamExt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::warn;
use url::Url;

/// Parses an invite token to extract the claims, including the remote server URL.
pub fn extract_token_claims(token: &str) -> Result<AlbumShareClaims> {
    decode::<AlbumShareClaims>(
        token,
        &DecodingKey::from_secret(settings().auth.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)
    .map_err(Into::into)
}

#[derive(Clone)]
pub struct S2SClient {
    http_client: Client,
}

impl S2SClient {
    pub const fn new(http_client: Client) -> Self {
        Self { http_client }
    }

    /// Fetches the summary of a shared album from a remote server using an invite token.
    pub async fn get_album_invite_summary(&self, token: &str) -> Result<AlbumSummary> {
        let claims = extract_token_claims(token)?;
        let remote_url = {
            let mut url: Url = claims.iss.parse()?;
            url.set_path("/s2s/albums/invite-summary");
            url
        };

        let response = self
            .http_client
            .get(remote_url.clone())
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(eyre!(
                "Remote server {remote_url} returned an error: {error_text}"
            ));
        }

        let summary: AlbumSummary = response.json().await?;
        Ok(summary)
    }

    pub async fn download_remote_file(
        &self,
        token: &str,
        remote_relative_path: &str,
        destination: &Path,
    ) -> Result<()> {
        let claims = extract_token_claims(token)?;
        let remote_url = {
            let mut url: Url = claims.iss.parse()?;
            url.set_path("/s2s/albums/files");
            url.query_pairs_mut()
                .append_pair("relativePath", remote_relative_path);
            url
        };
        let response = self
            .http_client
            .get(remote_url.clone())
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(eyre!(
                "Remote server returned an error during file download: {}, url: {}",
                error_text,
                remote_url.clone(),
            ));
        }

        let filename = response
            .headers()
            .get("content-disposition")
            .and_then(|val| val.to_str().ok())
            .and_then(|cd| cd.split("filename=").last())
            .map(|s| s.trim_matches('"').to_string())
            .ok_or_else(|| {
                eyre!("File from remote server {remote_url} does not have a filename header.")
            })?;

        if Some(filename)
            != destination
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
        {
            alert!(
                "WEIRD! Filename from S2S server {remote_url} does not match expected filename."
            );
        }

        // --- temp file ---
        let temp = NamedTempFile::new()?;
        let temp_path: PathBuf = temp.path().to_path_buf();
        let mut temp_file = fs::File::create(&temp_path).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            temp_file.write_all(&chunk?).await?;
        }
        temp_file.flush().await?;

        // --- move temp → destination ---
        fs::rename(&temp_path, &destination).await?;

        Ok(())
    }
}
