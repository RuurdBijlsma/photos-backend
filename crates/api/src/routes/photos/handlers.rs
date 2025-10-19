//! This module defines the HTTP handlers for general photos endpoints.

use crate::auth::db_model::User;
use crate::photos::interfaces::RandomPhotoResponse;
use crate::photos::service::get_random_photo;
use crate::setup::error::SetupError;
use crate::setup::interfaces::DiskResponse;
use axum::extract::State;
use axum::{Extension, Json};
use sqlx::PgPool;

/// Get random photo and associated theme.
///
/// # Errors
///
/// Returns a `PhotosError` if a configured media or thumbnail path is not a valid directory.
#[utoipa::path(
    get,
    path = "/photos/random",
    responses(
        (status = 200, description = "Get random photo and associated themes.", body = DiskResponse),
        (status = 500, description = "A configured path is not a valid directory"),
    )
)]
pub async fn random_photo(
    State(pool): State<PgPool>,
    Extension(user): Extension<User>,
) -> Result<Json<Option<RandomPhotoResponse>>, SetupError> {
    let result = get_random_photo(&user, &pool).await?;
    Ok(Json(result))
}
