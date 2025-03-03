use anyhow::{Error, Result};
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
    pub fn new(base_url: &str, endpoint: &'static str) -> Self {
        Self {
            http_client: Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .read_timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
            endpoint,
        }
    }

    pub async fn submit_job<T: Serialize>(&self, request: &T) -> Result<String, ApiClientError> {
        let url = format!("{}/{}", self.base_url, self.endpoint);
        let response = self.http_client.post(&url).json(request).send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let text = response.text().await?;
                Err(ApiClientError::UnexpectedStatus { status, text })
            }
        }
    }

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
