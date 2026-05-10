use crate::api_state::ApiContext;
use axum::extract::State;
use axum::{Extension, Json};
use common_services::api::system::interfaces::SystemStats;
use common_services::api::system::service::get_system_stats;
use common_services::api::user::error::UserError;
use common_services::database::app_user::User;

pub async fn get_system_stats_handler(
    State(ctx): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Json<SystemStats>, UserError> {
    let stats = get_system_stats(&ctx.pool, user.id).await?;
    Ok(Json(stats))
}
