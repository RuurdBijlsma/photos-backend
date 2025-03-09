use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Unexpected status {status}: {text}")]
    UnexpectedStatus { status: StatusCode, text: String },
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct ApiClient {
    http_client: Client,
    base_url: String,
    endpoint: &'static str,
}

impl ApiClient {
    /// Create api client
    ///
    /// # Panics
    /// if it can't create the client.
    #[must_use]
    pub fn new(base_url: &str, endpoint: &'static str) -> Self {
        Self {
            http_client: Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
            endpoint,
        }
    }

    /// Submit job to API
    ///
    /// # Errors
    /// * If POST request can't be made to url.
    /// * If json can't be parsed
    /// * If body can't be read
    /// * If unexpected status code is received.
    pub async fn submit_job<T: Serialize>(&self, request: T) -> Result<String, ApiClientError> {
        let url = format!("{}/{}", self.base_url, self.endpoint);
        let response = self.http_client.post(&url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(ApiClientError::UnexpectedStatus { status, text })
            }
        }
    }

    /// Check status of job via api
    ///
    /// # Errors
    /// * If GET request can't be made to url.
    /// * If json can't be parsed
    /// * If body can't be read
    /// * If unexpected status code is received.
    pub async fn check_status<J: DeserializeOwned>(
        &self,
        job_id: &str,
    ) -> Result<J, ApiClientError> {
        let url = format!("{}/{}/{}", self.base_url, self.endpoint, job_id);
        let response = self.http_client.get(&url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(ApiClientError::UnexpectedStatus { status, text })
            }
        }
    }

    /// Delete job from api, for when it's no longer needed.
    ///
    /// # Errors
    /// * If DELETE request can't be made to url.
    /// * If unexpected status code is received.
    /// * If body can't be read
    pub async fn delete_job(&self, job_id: &str) -> Result<(), ApiClientError> {
        let url = format!("{}/{}/{}", self.base_url, self.endpoint, job_id);
        let response = self.http_client.delete(&url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            status => {
                let text = response.text().await?;
                Err(ApiClientError::UnexpectedStatus { status, text })
            }
        }
    }
}
