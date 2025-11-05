use crate::auth::db_model::User;
use crate::pb::api::{ByMonthResponse, TimelineResponse};
use crate::photos::error::PhotosError;
use crate::photos::full_item_interfaces::FullMediaItem;
use crate::photos::interfaces::{GetMediaByMonthParams, GetMediaItemParams, RandomPhotoResponse};
use crate::photos::service::{
    fetch_full_media_item, get_photos_by_month, get_timeline_ids, get_timeline_ratios, random_photo,
};
use axum::extract::{Query, State};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use chrono::NaiveDate;
use sqlx::PgPool;

/// Get a random photo and its associated theme.
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
pub async fn get_full_item_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaItemParams>,
) -> Result<Json<FullMediaItem>, PhotosError> {
    let item = fetch_full_media_item(&user, &pool, &params.id).await?;
    if let Some(item) = item {
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
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Json<Option<RandomPhotoResponse>>, PhotosError> {
    let result = random_photo(&user, &pool).await?;
    Ok(Json(result))
}

/// Get a timeline of all media ratios, grouped by month.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/timeline/ratios",
    tag = "Photos",
    responses(
        (status = 200, description = "A timeline of media items grouped by month.", body = TimelineResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_timeline_ratios_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Protobuf<TimelineResponse>, PhotosError> {
    let timeline = get_timeline_ratios(&user, &pool).await?;
    Ok(Protobuf(timeline))
}

/// Get a timeline of all media ids
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/timeline/ids",
    tag = "Photos",
    responses(
        (status = 200, description = "A timeline of media items grouped by month.", body = Vec<String>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_timeline_ids_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<String>>, PhotosError> {
    let timeline = get_timeline_ids(&user, &pool).await?;
    Ok(Json(timeline))
}

/// Get all media items for a given set of months.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/by-month",
    tag = "Photos",
    params(
        GetMediaByMonthParams
    ),
    responses(
        (status = 200, description = "A collection of media items for the requested months.", body = ByMonthResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_photos_by_month_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaByMonthParams>,
) -> Result<Protobuf<ByMonthResponse>, PhotosError> {
    let month_ids = params
        .months
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d"))
        .collect::<Result<Vec<NaiveDate>, _>>()
        .map_err(|_| {
            PhotosError::InvalidMonthFormat(
                "Invalid date format in 'months' parameter. Please use 'YYYY-MM-DD'.".to_string(),
            )
        })?;
    let photos = get_photos_by_month(&user, &pool, &month_ids).await?;
    Ok(Protobuf(photos))
}
