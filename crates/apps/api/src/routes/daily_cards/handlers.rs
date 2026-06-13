use axum::extract::{Query, State};
use axum::{Extension, Json};
use common_services::database::app_user::User;
use crate::api_state::ApiContext;
use common_services::api::daily_cards::error::DailyCardsError;
use common_services::api::daily_cards::interfaces::{
    DailyCardResponse, DailyCardsQueryParams, ValidateMediaRequest,
};
use common_services::api::daily_cards::service::{get_daily_cards, validate_media_items};
use chrono::NaiveDate;
use tracing::instrument;

#[instrument(skip(context, user), err(Debug))]
pub async fn get_daily_cards_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<DailyCardsQueryParams>,
) -> Result<Json<Vec<DailyCardResponse>>, DailyCardsError> {
    let target_date = match params.date {
        Some(d_str) => NaiveDate::parse_from_str(&d_str, "%Y-%m-%d")
            .map_err(|_| DailyCardsError::BadRequest("Invalid date format. Expected YYYY-MM-DD".to_string()))?,
        None => chrono::Utc::now().naive_utc().date(),
    };

    let result = get_daily_cards(&context.pool, user.id, target_date, &context.settings).await?;
    Ok(Json(result))
}

#[instrument(skip(context, user), err(Debug))]
pub async fn validate_media_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Json(payload): Json<ValidateMediaRequest>,
) -> Result<Json<Vec<String>>, DailyCardsError> {
    let result = validate_media_items(&context.pool, user.id, &payload.media_item_ids).await?;
    Ok(Json(result))
}
