use crate::auth::db_model::User;
use crate::pb::api::{ByMonthResponse, TimelineResponse};
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{GetMediaByMonthParams, RandomPhotoResponse};
use crate::photos::service::{get_photos_by_month, get_timeline, random_photo};
use axum::extract::{Query, State};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use sqlx::PgPool;

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

/// Get a timeline of all media, grouped by month.
///
/// # Errors
///
/// Returns a `PhotosError` if the database query fails.
#[utoipa::path(
    get,
    path = "/photos/timeline",
    tag = "Photos",
    responses(
        (status = 200, description = "A timeline of media items grouped by month.", body = TimelineResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_timeline_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Protobuf<TimelineResponse>, PhotosError> {
    let timeline = get_timeline(&user, &pool).await?;
    Ok(Protobuf(timeline))
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
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let photos = get_photos_by_month(&user, &pool, &month_ids).await?;
    Ok(Protobuf(photos))
}
