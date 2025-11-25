use crate::api_state::ApiContext;
use crate::timeline::websocket::handle_timeline_socket;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use chrono::NaiveDate;
use common_services::api::timeline::error::TimelineError;
use common_services::api::timeline::interfaces::GetMediaByMonthParams;
use common_services::api::timeline::service::{
    get_photos_by_month, get_timeline_ids, get_timeline_ratios,
};
use common_services::database::app_user::User;
use common_types::pb::api::{ByMonthResponse, TimelineResponse};

/// Get a timeline of all media ratios, grouped by month.
///
/// # Errors
///
/// Returns a `TimelineError` if the database query fails.
#[utoipa::path(
    get,
    path = "/timeline/ratios",
    tag = "Timeline",
    responses(
        (status = 200, description = "A timeline of media items grouped by month.", body = TimelineResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_timeline_ratios_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Protobuf<TimelineResponse>, TimelineError> {
    let timeline = get_timeline_ratios(&user, &context.pool).await?;
    Ok(Protobuf(timeline))
}

/// Get a timeline of all media ids
///
/// # Errors
///
/// Returns a `TimelineError` if the database query fails.
#[utoipa::path(
    get,
    path = "/timeline/ids",
    tag = "Timeline",
    responses(
        (status = 200, description = "A timeline of media items grouped by month.", body = Vec<String>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_timeline_ids_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<String>>, TimelineError> {
    let timeline = get_timeline_ids(&user, &context.pool).await?;
    Ok(Json(timeline))
}

/// Get all media items for a given set of months.
///
/// # Errors
///
/// Returns a `TimelineError` if the database query fails.
#[utoipa::path(
    get,
    path = "/timeline/by-month",
    tag = "Timeline",
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
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<GetMediaByMonthParams>,
) -> Result<Protobuf<ByMonthResponse>, TimelineError> {
    let month_ids = params
        .months
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d"))
        .collect::<Result<Vec<NaiveDate>, _>>()
        .map_err(|_| {
            TimelineError::InvalidMonthFormat(
                "Invalid date format in 'months' parameter. Please use 'YYYY-MM-DD'.".to_string(),
            )
        })?;
    let photos = get_photos_by_month(&user, &context.pool, &month_ids).await?;
    Ok(Protobuf(photos))
}

/// Real-time timeline updates via WebSocket.
///
/// Requires `Sec-WebSocket-Protocol: access_token, <YOUR_JWT>` header.
#[utoipa::path(
    get,
    path = "/timeline/ws",
    tag = "Timeline",
    responses(
        (status = 101, description = "WebSocket upgrade")
    ),
    params(
        ("Sec-WebSocket-Protocol" = String, Header, description = "Auth: 'access_token, <JWT>'")
    )
)]
pub async fn timeline_websocket_handler(
    ws: WebSocketUpgrade,
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> axum::response::Response {
    ws.protocols(["access_token"])
        .on_upgrade(move |socket| handle_timeline_socket(socket, context, user))
}