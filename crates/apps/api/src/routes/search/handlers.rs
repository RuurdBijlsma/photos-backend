use crate::api_state::ApiContext;
use axum::Extension;
use axum::extract::{Query, State};
use axum_extra::protobuf::Protobuf;
use common_services::api::search::error::SearchError;
use common_services::api::search::interfaces::SearchParams;
use common_services::api::search::service::{
    SearchMediaConfig, get_search_suggestions, search_media,
};
use common_services::database::app_user::User;
use common_types::pb::api::{SearchResponse, SearchSuggestionsResponse};
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
        (status = 200, description = "Search results", body = Vec<SearchResponse>),
        (status = 500, description = "A database or internal error occurred."),
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(context, user), err(Debug))]
pub async fn get_search_results(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<SearchParams>,
) -> Result<Protobuf<SearchResponse>, SearchError> {
    let items = search_media(
        &user,
        &context.pool,
        context.embedder,
        &params.query,
        SearchMediaConfig {
            text_weight: context.settings.ingest.analyzer.search.text_weight,
            semantic_weight: context.settings.ingest.analyzer.search.semantic_weight,
            limit: params.limit,
        },
    )
    .await?;
    Ok(Protobuf(SearchResponse { items }))
}

#[instrument(skip(context, user), err(Debug))]
pub async fn get_search_suggestions_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
    Query(params): Query<SearchParams>,
) -> Result<Protobuf<SearchSuggestionsResponse>, SearchError> {
    let result = get_search_suggestions(
        &user,
        &context.pool,
        context.embedder,
        &params.query,
        SearchMediaConfig {
            text_weight: context.settings.ingest.analyzer.search.text_weight,
            semantic_weight: context.settings.ingest.analyzer.search.semantic_weight,
            limit: params.limit,
        },
    )
    .await?;
    Ok(Protobuf(result))
}
