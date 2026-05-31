use app_state::IngestSettings;
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use common_services::api::album::interfaces::MediaItemWithAlbums;
use common_services::api::photos::interfaces::{
    ColorThemeParams, DownloadMediaParams, GeoPhotosParams, RandomPhotoResponse,
    UpdateMediaItemRequest,
};
use common_services::api::photos::service::{
    download_media_file, random_photo, stream_video_file, thumbnail_on_demand_cached,
    update_media_item,
};
use common_services::database::album_store::AlbumStore;
use common_services::database::app_user::User;

use crate::api_state::ApiContext;
use axum::http::header;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum_extra::headers::Range;
use common_services::api::photos::error::PhotosError;
use common_services::api::photos::interfaces::PhotoThumbnailParams;
use common_services::database::media_item_store::MediaItemStore;
use common_types::pb::api::MapPhotosResponse;
use material_color_utils::utils::color_utils::Argb;
use material_color_utils::{MaterializedTheme, theme_from_color};
use tracing::instrument;

#[instrument(skip(context, user), err(Debug))]
pub async fn get_full_item_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(media_item_id): Path<String>,
) -> Result<Json<MediaItemWithAlbums>, PhotosError> {
    let item = MediaItemStore::find_by_id(&context.pool, &media_item_id).await?;
    if let Some(item) = item
        && item.user_id == user.id
    {
        Ok(Json(MediaItemWithAlbums {
            media_item: item,
            albums: AlbumStore::list_for_media_item(&context.pool, user.id, &media_item_id).await?,
        }))
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

pub async fn get_random_photo(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Json<Option<RandomPhotoResponse>>, PhotosError> {
    let result = random_photo(&user, &context.pool).await?;
    Ok(Json(result))
}

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

#[instrument(skip(context, user), err(Debug))]
pub async fn get_geo_photos_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<GeoPhotosParams>,
) -> Result<Protobuf<MapPhotosResponse>, PhotosError> {
    let items = MediaItemStore::find_all_geo_by_user_id(
        &context.pool,
        user.id,
        params.start_date,
        params.end_date,
    )
    .await?;
    Ok(Protobuf(MapPhotosResponse { items }))
}
