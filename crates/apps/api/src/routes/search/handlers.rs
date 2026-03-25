use crate::api_state::ApiContext;
use axum::extract::{Query, State};
use axum::{Extension, Json};
use axum_extra::protobuf::Protobuf;
use common_services::api::search::error::SearchError;
use common_services::api::search::interfaces::{
    SearchFilterRanges, SearchMediaConfig, SearchParams,
};
use common_services::api::search::service::{
    get_random_search_suggestion, get_search_suggestions, search_filter_ranges, search_media,
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
            embedder_model_id: context.settings.ingest.analyzer.search.embedder_model_id.clone(),
            semantic_score_threshold: context
                .settings
                .ingest
                .analyzer
                .search
                .semantic_score_threshold,
            text_weight: context.settings.ingest.analyzer.search.text_weight,
            semantic_weight: context.settings.ingest.analyzer.search.semantic_weight,
            limit: params.limit,
            start_date: params.start_date,
            end_date: params.end_date,
            media_type: params.media_type,
            sort_by: params.sort_by,
            negative_query: params.negative_query,
            country_codes: params
                .country_codes
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            face_name: params.face_name,
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
    let result = get_search_suggestions(&user, &context.pool, &params.query, params.limit).await?;
    Ok(Protobuf(result))
}

#[instrument(skip(context, user), err(Debug))]
pub async fn get_random_search_suggestion_handler(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<String, SearchError> {
    let result = get_random_search_suggestion(&user, &context.pool).await?;
    Ok(result.unwrap_or_default())
}

#[instrument(skip(context, user), err(Debug))]
pub async fn get_search_filter_ranges(
    State(context): State<ApiContext>,
    Extension(user): Extension<User>,
) -> Result<Json<SearchFilterRanges>, SearchError> {
    let result = search_filter_ranges(&user, &context.pool).await?;
    Ok(Json(result))
}
