use crate::auth::db_model::User;
use crate::pb::api::{GetMonthlyRatiosResponse, MonthlyRatios};
use crate::pb::api::MonthGroup;
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    GetByMonthParam, GetMediaByMonthParams, MonthlyRatiosDto,
    PaginatedMediaResponse, RandomPhotoResponse, TimelineSummary,
};
use crate::photos::service::{
    get_all_photo_ratios2, get_media_by_month, get_media_by_months, get_timeline_summary,
    random_photo,
};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use http::{header, HeaderMap};
use prost::Message;
use sqlx::PgPool;
use std::time::Instant;

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

#[utoipa::path(
    get,
    path = "/photos/by-month.pb",
    tag = "Photos",
    params(
        GetByMonthParam
    ),
    responses(
        (status = 200, description = "Get media items for the requested months.", body = MonthGroup),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[axum::debug_handler]
pub async fn get_media_by_month_protobuf_handler(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
    Query(params): Query<GetByMonthParam>,
) -> Result<Protobuf<MonthGroup>, PhotosError> {
    // 1. Call your existing function to get the DTO response
    let mg = get_media_by_month(&params.month, &user, &pool).await?;
    Ok(Protobuf(mg))
}

/// Get all photo ratios grouped by month in json format.
#[utoipa::path(
    get,
    path = "/photos/ratios",
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
) -> Result<Json<Vec<MonthlyRatiosDto>>, PhotosError> {
    let now = Instant::now();
    let ratios_by_month = get_all_photo_ratios2(&user, &pool).await?;
    println!("service took {:?}", now.elapsed());
    Ok(Json(ratios_by_month))
}

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
    let ratios_by_month: Vec<MonthlyRatiosDto> = get_all_photo_ratios2(&user, &pool).await?;

    let response = GetMonthlyRatiosResponse {
        results: ratios_by_month
            .into_iter()
            .map(|db_result| MonthlyRatios {
                month: db_result.month,
                ratios: db_result.ratios,
            })
            .collect(),
    };

    let body = response.encode_to_vec();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-protobuf".parse().unwrap(),
    );
    Ok((headers, body).into_response())
}
