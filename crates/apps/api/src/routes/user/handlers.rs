use crate::api_state::ApiContext;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_services::api::user::error::UserError;
use common_services::api::user::interfaces::{UpdateUserProfileRequest, UserProfile};
use common_services::api::user::service::{get_user_profile, update_user_profile};
use common_services::database::app_user::User;

/// Fetch the profile data and library statistics for any user.
#[utoipa::path(
    get,
    path = "/user/{user_id}/profile",
    responses(
        (status = 200, description = "Profile data and stats fetched successfully", body = UserProfile),
        (status = 404, description = "User not found")
    ),
    params(
        ("user_id" = i32, Path, description = "User ID")
    ),
    tag = "User",
    security(("bearer_auth" = []))
)]
pub async fn get_user_profile_handler(
    State(ctx): State<ApiContext>,
    Extension(user): Extension<User>,
    Path(user_id): Path<i32>,
) -> Result<Json<UserProfile>, UserError> {
    let profile = get_user_profile(&ctx.pool, user.id, user_id).await?;
    Ok(Json(profile))
}

/// Update the current authenticated user's settings.
#[utoipa::path(
    put,
    path = "/user/profile",
    responses(
        (status = 200, description = "Profile updated successfully", body = UserProfile),
        (status = 400, description = "Invalid input or avatar item")
    ),
    request_body = UpdateUserProfileRequest,
    tag = "User",
    security(("bearer_auth" = []))
)]
pub async fn update_my_profile(
    State(ctx): State<ApiContext>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateUserProfileRequest>,
) -> Result<Json<UserProfile>, UserError> {
    let profile = update_user_profile(&ctx.pool, user.id, payload.name, payload.avatar_id).await?;
    Ok(Json(profile))
}
