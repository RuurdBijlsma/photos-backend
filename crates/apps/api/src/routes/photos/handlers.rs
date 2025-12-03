use crate::api_state::ApiContext;
use app_state::IngestSettings;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use common_services::api::photos::error::PhotosError;
use common_services::api::photos::interfaces::{
    ColorThemeParams, DownloadMediaParams, GetMediaItemParams, RandomPhotoResponse,
};
use common_services::api::photos::service::{download_media_file, random_photo};
use common_services::database::app_user::User;
use common_services::database::media_item::media_item::FullMediaItem;
use common_services::database::media_item_store::MediaItemStore;
use ml_analysis::get_color_theme;
use serde_json::Value;
use tracing::instrument;

/// Get a full media item
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/item",
    tag = "Photos",
    params(
        GetMediaItemParams
    ),
    responses(
        (status = 200, description = "Get random photo and associated themes.", body = FullMediaItem),
        (status = 404, description = "Item not found."),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn get_full_item_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaItemParams>,
) -> Result<Json<FullMediaItem>, PhotosError> {
    let item = MediaItemStore::find_by_id(&context.pool, &params.id).await?;
    if let Some(item) = item
        && item.user_id == user.id
    {
        Ok(Json(item))
    } else {
        Err(PhotosError::MediaNotFound(params.id))
    }
}

/// Get a random photo and its associated theme.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/random",
    tag = "Photos",
    responses(
        (status = 200, description = "Get random photo and associated themes.", body = RandomPhotoResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_random_photo(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Json<Option<RandomPhotoResponse>>, PhotosError> {
    let result = random_photo(&user, &context.pool).await?;
    Ok(Json(result))
}

/// Get a random photo and its associated theme.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/theme",
    tag = "Photos",
    params(
        ColorThemeParams
    ),
    responses(
        (status = 200, description = "Get theme object from a color.", body = Value),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_color_theme_handler(
    State(ingestion): State<IngestSettings>,
    Query(params): Query<ColorThemeParams>,
) -> Result<Json<Value>, PhotosError> {
    let variant = &ingestion.analyzer.theme_generation.variant;
    let contrast_level = ingestion.analyzer.theme_generation.contrast_level;
    Ok(Json(get_color_theme(
        &params.color,
        variant,
        contrast_level as f32,
    )?))
}

/// Download a media file.
///
/// This endpoint streams a specific media file to the client. The path to the media
/// file must be a valid and secure path within the configured media directory.
///
/// # Errors
///
/// This function returns a `PhotosError` if the path is invalid, the file
/// isn't found, the user lacks permissions, or an internal server error occurs.
#[utoipa::path(
    get,
    path = "/photos/download",
    tag = "Photos",
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
    State(ingestion): State<IngestSettings>,
    Extension(user): Extension<User>,
    Query(query): Query<DownloadMediaParams>,
) -> Result<impl IntoResponse, PhotosError> {
    let response = download_media_file(&ingestion, &user, &query.path).await?;
    Ok(response)
}
