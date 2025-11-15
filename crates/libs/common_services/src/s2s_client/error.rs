use thiserror::Error;

#[derive(Error, Debug)]
pub enum S2sClientError {
    #[error("Invalid token: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Failed to build request URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Remote server returned an error: {0}")]
    RemoteServerError(String),
}