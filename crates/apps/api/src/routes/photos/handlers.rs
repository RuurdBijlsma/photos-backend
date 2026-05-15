use app_state::IngestSettings;
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use common_services::api::photos::interfaces::{
    ColorThemeParams, DownloadMediaParams, RandomPhotoResponse, UpdateMediaItemRequest,
};
use common_services::api::photos::service::{
    download_media_file, random_photo, stream_video_file, thumbnail_on_demand_cached,
    update_media_item,
};
use common_services::database::app_user::User;
use common_services::database::media_item::media_item::FullMediaItem;

use crate::api_state::ApiContext;
use axum::http::header;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum_extra::headers::Range;
use common_services::api::photos::error::PhotosError;
use common_services::api::photos::interfaces::PhotoThumbnailParams;
use common_services::database::media_item_store::MediaItemStore;
use material_color_utils::utils::color_utils::Argb;
use material_color_utils::{MaterializedTheme, theme_from_color};
use tracing::instrument;

#[instrument(skip(context, user), err(Debug))]
pub async fn get_full_item_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(media_item_id): Path<String>,
) -> Result<Json<FullMediaItem>, PhotosError> {
    let item = MediaItemStore::find_by_id(&context.pool, &media_item_id).await?;
    if let Some(item) = item
        && item.user_id == user.id
    {
        Ok(Json(item))
    } else {
        Err(PhotosError::MediaNotFound(media_item_id))
    }
}

#[instrument(skip(context, user), err(Debug))]
pub async fn update_media_item_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(media_item_id): Path<String>,
    Json(payload): Json<UpdateMediaItemRequest>,
) -> Result<(), PhotosError> {
    update_media_item(&context.pool, &media_item_id, user.id, &payload).await?;

    Ok(())
}

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
) -> Result<Json<MaterializedTheme>, PhotosError> {
    let variant = &ingestion.analyzer.theme_generation.variant;
    let contrast_level = ingestion.analyzer.theme_generation.contrast_level;
    let theme = theme_from_color(Argb::from_hex(&params.color)?)
        .variant(*variant)
        .contrast_level(contrast_level)
        .call();
    Ok(Json(theme))
}

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
pub async fn download_full_file_by_rel_path(
    State(ingestion): State<IngestSettings>,
    Extension(user): Extension<User>,
    Query(query): Query<DownloadMediaParams>,
) -> Result<impl IntoResponse, PhotosError> {
    let response = download_media_file(&ingestion, &user, &query.path).await?;
    Ok(response)
}

pub async fn download_full_file_by_id(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(media_item_id): Path<String>,
) -> Result<impl IntoResponse, PhotosError> {
    let Some(rel_path) =
        MediaItemStore::find_relative_path_by_id(&context.pool, &media_item_id).await?
    else {
        return Err(PhotosError::MediaNotFound(media_item_id));
    };
    let response = download_media_file(&context.settings.ingest, &user, &rel_path).await?;
    Ok(response)
}

#[instrument(skip(context), err(Debug))]
pub async fn get_photo_thumbnail(
    State(context): State<ApiContext>,
    Query(query): Query<PhotoThumbnailParams>,
    Path(media_item_id): Path<String>,
) -> Result<impl IntoResponse, PhotosError> {
    let size = query.size.unwrap_or(360);
    if size > 1440 {
        return Err(PhotosError::AccessDenied);
    }

    let image_bytes = thumbnail_on_demand_cached(
        &context.pool,
        size,
        &media_item_id,
        &context.settings.ingest,
    )
    .await?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        image_bytes,
    ))
}

#[utoipa::path(
    get,
    path = "/photos/video/{media_item_id}",
    tag = "Photos",
    responses((status = 206, description = "Partial video content"))
)]
pub async fn stream_video_handler(
    State(context): State<ApiContext>,
    Path(media_item_id): Path<String>,
    range: Option<TypedHeader<Range>>,
) -> Result<impl IntoResponse, PhotosError> {
    let range_inner = range.map(|TypedHeader(r)| r);
    stream_video_file(
        &context.pool,
        &context.settings.ingest,
        &media_item_id,
        range_inner,
    )
    .await
}
