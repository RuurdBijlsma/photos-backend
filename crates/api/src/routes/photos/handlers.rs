use crate::auth::db_model::User;
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{GetMediaByMonthParams, PaginatedMediaResponse, RandomPhotoResponse, TimelineSummary};
use crate::photos::service::{get_media_by_months, get_timeline_summary, random_photo};
use axum::extract::{Query, State};
use axum::{Extension, Json};
use sqlx::PgPool;

/// Get random photo and associated theme.
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
/// Get a summary of media counts by month and year.
#[utoipa::path(
    get,
    path = "/photos/timeline",
    tag = "Photos",
    responses(
        (status = 200, description = "Get a summary of media counts by month and year.", body = Vec<TimelineSummary>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_timeline_summary_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<TimelineSummary>>, PhotosError> {
    let summary = get_timeline_summary(&user, &pool).await?;
    Ok(Json(summary))
}

/// Get media items for a given set of months, grouped by day.
#[utoipa::path(
    get,
    path = "/photos/by-month",
    tag = "Photos",
    params(
        GetMediaByMonthParams
    ),
    responses(
        (status = 200, description = "Get media items for the requested months.", body = PaginatedMediaResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_media_by_month_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaByMonthParams>,
) -> Result<Json<PaginatedMediaResponse>, PhotosError> {
    let media = get_media_by_months(&params, &user, &pool).await?;
    Ok(Json(media))
}
