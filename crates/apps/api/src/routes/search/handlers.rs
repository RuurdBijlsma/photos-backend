use crate::api_state::ApiContext;
use axum::extract::{Query, State};
use axum::{Extension, Json};
use common_services::api::search::error::SearchError;
use common_services::api::search::interfaces::{SearchParams, SearchResponse};
use common_services::api::search::service::search_media;
use common_services::database::app_user::User;

/// Get a timeline of all media ratios, grouped by month.
///
/// # Errors
///
/// Returns a `TimelineError` if the database query fails.
#[utoipa::path(
    get,
    path = "/search",
    tag = "Search",
    params(
        SearchParams
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[axum::debug_handler]
pub async fn get_search_results(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, SearchError> {
    let search_result = search_media(&user, &context.pool, &params.query).await?;
    Ok(Json(search_result))
}
