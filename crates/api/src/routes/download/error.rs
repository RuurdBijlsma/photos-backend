use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use color_eyre::eyre;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Invalid media path provided.")]
    InvalidPath,

    #[error("The requested media file was not found.")]
    NotFound,

    #[error("Permission denied while accessing media file.")]
    AccessDenied,

    #[error("The requested file is not a supported media type.")]
    UnsupportedMediaType,

    #[error("An internal error occurred.")]
    Internal(#[from] eyre::Report),
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response<Body> {
        let (status, error_message) = match self {
            Self::InvalidPath => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::AccessDenied => (StatusCode::FORBIDDEN, self.to_string()),
            Self::UnsupportedMediaType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string()),
            Self::Internal(ref e) => {
                // Log the full error for debugging purposes
                error!("Internal media error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.".to_string(),
                )
            }
        };

        (status, error_message).into_response()
    }
}
