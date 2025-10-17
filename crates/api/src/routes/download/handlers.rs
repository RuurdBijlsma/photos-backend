use axum::Extension;
use axum::extract::Query;
use axum::response::IntoResponse;
use crate::auth::db_model::User;
use crate::download::error::DownloadError;
use crate::download::interfaces::DownloadMediaQuery;
use crate::download::service::download_media_file;

/// Download a media file.
///
/// This endpoint streams a specific media file to the client. The path to the media
/// file must be a valid and secure path within the configured media directory.
#[utoipa::path(
    get,
    path = "/download/full-file",
    params(
        ("path" = String, Query, description = "The path of the media file to download")
    ),
    responses(
        (status = 200, description = "Media file streamed successfully.", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 400, description = "Invalid path provided, such as a directory traversal attempt."),
        (status = 403, description = "Permission denied when trying to access the file."),
        (status = 404, description = "The requested media file could not be found."),
        (status = 415, description = "The requested file is not a supported media type."),
        (status = 500, description = "An internal server error occurred."),
    )
)]
pub async fn download_full_file(
    Extension(user): Extension<User>,
    Query(query): Query<DownloadMediaQuery>,
)->Result<impl IntoResponse, DownloadError>{
    let response = download_media_file(&user, &query.path).await?;
    Ok(response)
}
