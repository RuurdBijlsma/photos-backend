use crate::api_state::ApiContext;
use axum::extract::{Query, State};
use axum::{Extension, Json};
use common_services::api::search::error::SearchError;
use common_services::api::search::interfaces::{SearchParams, SearchResultItem};
use common_services::api::search::service::{SearchMediaConfig, search_media};
use common_services::database::app_user::User;
use tracing::instrument;

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
        (status = 200, description = "Search results", body = Vec<SearchResultItem>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn get_search_results(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResultItem>>, SearchError> {
    let search_result = search_media(
        &user,
        &context.pool,
        &context.embedder,
        &params.query,
        SearchMediaConfig {
            text_weight: context.settings.ingest.analyzer.search.text_weight,
            semantic_weight: context.settings.ingest.analyzer.search.semantic_weight,
            limit: params.limit,
            threshold: params.threshold,
        },
    )
    .await?;
    Ok(Json(search_result))
}
