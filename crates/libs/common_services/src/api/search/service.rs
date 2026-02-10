use crate::api::search::error::SearchError;
use crate::api::search::interfaces::SearchResultItem;
use crate::database::app_user::User;
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;

pub async fn search_media(
    user: &User,
    pool: &PgPool,
    query: &str,
    requested_limit: Option<i64>,
    threshold: Option<f32>,
    embedder: &TextEmbedder,
) -> Result<Vec<SearchResultItem>, SearchError> {
    let query_embedding = embedder.embed_text(query)?.to_vec();
    let vector_param = Vector::from(query_embedding);

    let limit = requested_limit.unwrap_or(50).min(100);
    let cutoff = threshold.unwrap_or(0.0);
    // todo: set weights from settings
    // todo: set language from settings?

    // We fetch more candidates than the limit to ensure that after
    // the combined score calculation and thresholding, we still have results.
    let candidate_limit = limit * 3;

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
            mi.id,
            mi.is_video,
            mi.use_panorama_viewer as is_panorama,
            mi.duration_ms,
            mi.taken_at_local,
            (mi.width::real / mi.height::real) as ratio,
            coalesce(f.rank, 0)::real as fts_score,
            coalesce(v.similarity, 0)::real as vector_score,
            (coalesce(f.rank, 0) * 0.4 + coalesce(v.similarity, 0) * 0.6)::real as combined_score
        FROM media_item mi
        LEFT JOIN fts_search f ON mi.id = f.id
        LEFT JOIN vector_search v ON mi.id = v.id
        WHERE (f.id IS NOT NULL OR v.id IS NOT NULL)
          AND (coalesce(f.rank, 0) * 0.4 + coalesce(v.similarity, 0) * 0.6) >= $6
        ORDER BY mi.sort_timestamp DESC
        LIMIT $5
        ",
    )
    .bind(query)
    .bind(user.id)
    .bind(vector_param)
    .bind(candidate_limit)
    .bind(limit)
    .bind(cutoff)
    .fetch_all(pool)
    .await?;

    Ok(items)
}
