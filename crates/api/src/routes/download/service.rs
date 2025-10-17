use axum::{
    body::Body,
    http::{header, StatusCode},
};
use color_eyre::Report;
use common_photos::{
    is_media_file, media_dir, relative_path_canon,
};
use http::Response;
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{debug, warn};
use crate::auth::db_model::{User, UserRole};
use crate::download::error::DownloadError;

pub async fn download_media_file(user: &User, path: &str) -> Result<Response<Body>, DownloadError> {
    debug!("download_full_file called for path: {}", path);

    // --- 1. Security & Path Validation ---
    let media_dir_canon = media_dir()
        .canonicalize()
        .map_err(|e| Report::new(e).wrap_err("Failed to canonicalize media directory"))?;

    let file_path = media_dir().join(path);
    let file_canon = match file_path.canonicalize() {
        Ok(path) => path,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("File not found at path: {}", file_path.display());
            return Err(DownloadError::NotFound);
        }
        Err(e) => return Err(Report::new(e).wrap_err("Failed to canonicalize path").into()),
    };

    if !file_canon.starts_with(&media_dir_canon) {
        warn!("Blocked directory traversal attempt for: {}", path);
        return Err(DownloadError::InvalidPath);
    }

    // --- 2. Authorization ---
    let relative_path = relative_path_canon(&file_canon)?;
    if user.role != UserRole::Admin {
        let Some(user_media_folder) = &user.media_folder else {
            warn!("Access denied for user {}: No media folder assigned.", user.id);
            return Err(DownloadError::AccessDenied);
        };
        if !relative_path.starts_with(user_media_folder) {
            warn!("Access denied for user {}: Attempted to access path outside their media folder.", user.id);
            return Err(DownloadError::AccessDenied);
        }
    }

    // --- 3. Media Type Validation ---
    if !is_media_file(&file_canon) {
        warn!("Unsupported media type requested: {}", file_canon.display());
        return Err(DownloadError::UnsupportedMediaType);
    }

    // --- 4. File Access ---
    let file = match File::open(&file_canon).await {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Err(DownloadError::NotFound),
            std::io::ErrorKind::PermissionDenied => Err(DownloadError::AccessDenied),
            _ => Err(Report::new(e).wrap_err("Failed to open media file").into()),
        }?,
    };

    // --- 5. Build the Streaming Response ---
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::from_stream(stream);
    let mime_type = mime_guess::from_path(&file_canon).first_or_octet_stream();
    let filename = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("mediafile");
    let disposition = format!("inline; filename=\"{filename}\"");
    let disposition_header = header::HeaderValue::from_str(&disposition).unwrap_or(header::HeaderValue::from_static("inline"));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type.as_ref())
        .header(header::CONTENT_DISPOSITION, disposition_header)
        .body(body)
        .map_err(|e| Report::new(e).wrap_err("Failed to build response"))?)
}
