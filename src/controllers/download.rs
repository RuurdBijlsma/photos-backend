use path_clean::clean;
use std::path::Path;
use tracing::{debug, error, warn};

use crate::common::image_utils::{is_photo_file, is_video_file};
use crate::common::settings::Settings;
use axum::extract::Query;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
};
use loco_rs::prelude::*;
use mime_guess;
use serde::Deserialize;
use tokio::{fs::File, io::ErrorKind};
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Debug, Deserialize)]
pub struct MediaQuery {
    path: String,
}

/// # Errors
///
/// - **`Error::NotFound`**:
///   - The requested file does not exist.
///   - The provided path attempts directory traversal (security violation).
///
/// - **`Error::BadRequest`**:
///   - The requested file is not a supported media type (not a photo or video).
///
/// - **`Error::Unauthorized`**:
///   - The server does not have permission to access the requested file.
///
/// - **`Error::InternalServerError`**:
///   - An unexpected error occurred while opening or streaming the file.
pub async fn get_media(
    _: auth::JWT,
    Query(query): Query<MediaQuery>,
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse, Error> {
    debug!("get_media called for path: {}", query.path);

    let settings = Settings::from_context(&ctx);
    let media_dir = Path::new(&settings.media_dir);
    let cleaned_path = clean(media_dir.join(&query.path));

    // Security validation
    if !cleaned_path.starts_with(media_dir) {
        warn!("Blocked directory traversal attempt: {}", query.path);
        return Err(Error::NotFound);
    }

    // Media type validation
    if !(is_photo_file(&cleaned_path) || is_video_file(&cleaned_path)) {
        warn!("Unsupported media type: {}", cleaned_path.display());
        return Err(Error::BadRequest("Unsupported media type".into()));
    }

    // File handling with proper error mapping
    let file = File::open(&cleaned_path).await.map_err(|e| {
        error!("File open error: {}", e);
        match e.kind() {
            ErrorKind::NotFound => {
                debug!("File not found: {}", cleaned_path.display());
                Error::NotFound
            }
            ErrorKind::PermissionDenied => {
                warn!("Permission denied: {}", cleaned_path.display());
                Error::Unauthorized("Can't open file, permission denied.".to_string())
            }
            _ => Error::InternalServerError,
        }
    })?;

    // Determine MIME type based on file extension
    let mime_type = mime_guess::from_path(&cleaned_path)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

    // Streaming response
    let stream = FramedRead::new(file, BytesCodec::new());
    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type.to_string())
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", cleaned_path.display()),
        )
        .body(Body::from_stream(stream))?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/download")
        .add("/media", get(get_media))
}
