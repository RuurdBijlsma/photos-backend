// crates/api/src/routes/photos/handlers.rs

//! This module defines the HTTP handlers for general photos endpoints.

use crate::auth::db_model::User;
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    GetMediaByDateParams, GetMediaParams, PaginatedMediaResponse, RandomPhotoResponse,
};
use crate::photos::service::{media_by_date, media_paginated, random_photo};
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

/// Get a paginated list of media items.
///
/// Use `before` timestamp to get older items (scrolling down).
/// Use `after` timestamp to get newer items (scrolling up).
/// If neither is provided, returns the latest items.
#[utoipa::path(
    get,
    path = "/photos/media",
    tag = "Photos",
    params(GetMediaParams),
    responses(
        (status = 200, description = "Successfully retrieved media items.", body = PaginatedMediaResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_media(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaParams>,
) -> Result<Json<PaginatedMediaResponse>, PhotosError> {
    let result = media_paginated(&user, &pool, params).await?;
    Ok(Json(result))
}

/// Jump to a specific date in the media timeline.
///
/// Fetches a 'window' of media items centered around the provided date,
/// allowing the client to scroll up and down from that point.
#[utoipa::path(
    get,
    path = "/photos/media-by-date",
    tag = "Photos",
    params(GetMediaByDateParams),
    responses(
        (status = 200, description = "Successfully retrieved media items around the specified date.", body = PaginatedMediaResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_media_by_date(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaByDateParams>,
) -> Result<Json<PaginatedMediaResponse>, PhotosError> {
    let result = media_by_date(&user, &pool, params).await?;
    Ok(Json(result))
}
