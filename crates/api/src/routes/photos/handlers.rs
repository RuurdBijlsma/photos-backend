use std::time::Instant;
use crate::auth::db_model::User;
use crate::pb::api::{AllPhotoRatiosResponse, MonthlyPhotoRatios};
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    GetMediaByMonthParams, PaginatedMediaResponse, RandomPhotoResponse, TimelineSummary,
};
use crate::photos::service::{
    get_all_photo_ratios, get_media_by_months, get_timeline_summary, random_photo,
};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use http::{header, HeaderMap};
use prost::Message;
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

/// Get all photo ratios grouped by month in json format.
#[utoipa::path(
    get,
    path = "/photos/ratios.pb",
    tag = "Photos",
    responses(
        (status = 200, description = "TODO EDIT ME.", body = Vec<Vec<f32>>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_photo_ratios_json_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<Vec<f32>>>, PhotosError> {
    let now = Instant::now();
    let result = get_all_photo_ratios(&user, &pool).await?;
    println!("service took {:?}", now.elapsed());
    Ok(Json(result))
}

/// Get all photo ratios grouped by month in a compact Protobuf format.
#[utoipa::path(
    get,
    path = "/photos/ratios.pb",
    tag = "Photos",
    responses(
        (status = 200, description = "TODO EDIT ME.", body = Vec<Vec<f32>>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_photo_ratios_pb_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Response, PhotosError> {
    // 1. Fetch the data from the database, same as before.
    let ratios_by_month: Vec<Vec<f32>> = get_all_photo_ratios(&user, &pool).await?;

    // 2. Convert the Rust Vecs into the generated Protobuf structs.
    let response = AllPhotoRatiosResponse {
        months: ratios_by_month
            .into_iter()
            .map(|ratios| MonthlyPhotoRatios { ratios })
            .collect(),
    };

    // 3. Serialize the Protobuf struct into a binary Vec<u8>.
    let body = response.encode_to_vec();

    // 4. Create headers to specify the binary content type.
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-protobuf".parse().unwrap(),
    );

    // 5. Return the headers and binary body.
    Ok((headers, body).into_response())
}
