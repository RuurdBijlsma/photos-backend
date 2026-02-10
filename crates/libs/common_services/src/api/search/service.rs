use crate::api::search::error::SearchError;
use crate::api::search::interfaces::{SearchResponse, SearchResultItem};
use crate::database::app_user::User;
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;

pub async fn search_media(
    user: &User,
    pool: &PgPool,
    query: &str,
    requested_limit: Option<i64>,
    embed_model_id: &str,
) -> Result<SearchResponse, SearchError> {
    // todo: make this return vector similarity, FTS similarity, and total similarity for debug purposes
    // todo: make this return more complete items (width/height/id,etc.)
    // todo: sort this by time sort column? that's how google photos does it.
    // todo: make quick UI to show results, along with the three scores
    // todo: don't create text embedder here every time lol
    let embedder = TextEmbedder::from_hf(embed_model_id).build().await?;
    let query_embedding = embedder.embed_text(query)?.to_vec();
    let vector_param = Vector::from(query_embedding);
    let limit = requested_limit.unwrap_or(50).min(100);

    // Internal "Candidate" limit. We pull more candidates than requested
    // to ensure the join has enough data to find the best hybrid matches.
    let candidate_limit = limit * 2;
    let items = sqlx::query_as::<_, SearchResultItem>(
        r"
        WITH fts_search AS (
            SELECT
                id,
                ts_rank_cd(search_vector, websearch_to_tsquery('english', $1)) as rank
            FROM media_item
            WHERE user_id = $2
              AND search_vector @@ websearch_to_tsquery('english', $1)
              AND deleted = false
            ORDER BY rank DESC
            LIMIT $4
        ),
        vector_search AS (
            SELECT
                media_item_id as id,
                1 - (embedding <=> $3::vector) as similarity
            FROM visual_analysis
            JOIN media_item mi ON visual_analysis.media_item_id = mi.id
            WHERE mi.user_id = $2 AND mi.deleted = false
            ORDER BY embedding <=> $3::vector
            LIMIT $4
        )
        SELECT
            coalesce(f.id, v.id) as id,
            (coalesce(f.rank, 0) * 0.4 + coalesce(v.similarity, 0) * 0.6)::real as score
        FROM fts_search f
        FULL OUTER JOIN vector_search v ON f.id = v.id
        ORDER BY score DESC
        LIMIT $5
        ",
    )
    .bind(query)
    .bind(user.id)
    .bind(vector_param)
    .bind(candidate_limit)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(SearchResponse { items })
}
