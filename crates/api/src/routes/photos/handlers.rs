use crate::auth::db_model::User;
use crate::pb::api::{GetMonthlyRatiosResponse, MonthlyRatios};
use crate::photos::error::PhotosError;
use crate::photos::interfaces::{
    GetMediaByMonthParams, MonthlyRatiosDto, PaginatedMediaResponse, RandomPhotoResponse,
};
use crate::photos::service::{get_all_photo_ratios2, get_media_by_months, random_photo};
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
    path = "/photos/ratios.pb",
    tag = "Photos",
    responses(
        (status = 200, description = "TODO EDIT ME.", body = Vec<Vec<f32>>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
//todo rewrite function with Protobuf, use straight up pb types in db query, remove possible panics.
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
