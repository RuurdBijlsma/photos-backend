use crate::api::search::interfaces::SearchResponse;
use crate::database::app_user::User;
use sqlx::PgPool;
use crate::api::search::error::SearchError;

/// Fetches a timeline of media item ratios, grouped by month.
pub async fn search_media(
    user: &User,
    pool: &PgPool,
    query: &str,
) -> Result<SearchResponse, SearchError> {
    Ok(SearchResponse {
        result: "Hi".to_owned(),
    })
}
