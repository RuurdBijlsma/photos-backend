use crate::api_state::ApiState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use common_services::api::s2s::error::S2SError;
use common_services::api::s2s::service::{
    get_invite_summary, get_media_item_path, validate_token_for_media_item,
};
use tokio_util::io::ReaderStream;

pub async fn invite_summary_handler(
    State(api_state): State<ApiState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, S2SError> {
    let token = authorization.token();
    let summary = get_invite_summary(&api_state.pool, token).await?;
    Ok(Json(summary))
}

pub async fn download_file_handler(
    State(api_state): State<ApiState>,
    Path(media_item_id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, S2SError> {
    // The full token is passed in the bearer token
    let token = authorization.token();
    validate_token_for_media_item(&api_state.pool, token, &media_item_id).await?;

    let file_path = get_media_item_path(&api_state.pool, &media_item_id).await?;
    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Guess the MIME type from the file path.
    let mime_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .as_ref()
        .to_string();

    let file = tokio::fs::File::open(&file_path).await.map_err(|_| {
        S2SError::NotFound(format!(
            "File not found on disk for item {}",
            &media_item_id
        ))
    })?;

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    // Use the dynamically determined MIME type in the response header.
    let headers = [
        (header::CONTENT_TYPE, mime_type),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{file_name}\""),
        ),
    ];

    Ok((headers, body))
}
