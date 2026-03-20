use crate::api::search::error::SearchError;
use crate::database::app_user::User;
use common_types::pb::api::{SearchSuggestionsResponse, SimpleTimelineItem};
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;
use std::sync::Arc;

pub struct SearchMediaConfig {
    pub limit: Option<i64>,
    pub semantic_weight: f64,
    pub text_weight: f64,
}

pub async fn search_media(
    user: &User,
    pool: &PgPool,
    embedder: Arc<TextEmbedder>,
    query: &str,
    config: SearchMediaConfig,
) -> Result<Vec<SimpleTimelineItem>, SearchError> {
    let query_str = query.to_string();
    let embedder = embedder.clone();
    let query_embedding = tokio::task::spawn_blocking(move || embedder.embed_text(&query_str))
        .await??
        .to_vec();
    let vector_param = Vector::from(query_embedding);

    let limit = config.limit.unwrap_or(100).min(500);
    let candidate_limit = limit * 3 + 300;
    let k = 60.0f64;

    let items = sqlx::query_as!(
        SimpleTimelineItem,
        r#"
        WITH
        fts AS (
            SELECT
                id,
                ts_rank_cd(search_vector, websearch_to_tsquery('english', $1)) as score,
                ROW_NUMBER() OVER (ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('english', $1)) DESC) as rank
            FROM media_item
            WHERE user_id = $2
              AND search_vector @@ websearch_to_tsquery('english', $1)
              AND deleted = false
            LIMIT $4
        ),
        vec AS (
            SELECT
                id,
                1 - distance as score,
                ROW_NUMBER() OVER (ORDER BY distance) as rank
            FROM (
                SELECT DISTINCT ON (media_item_id)
                    media_item_id as id,
                    distance
                FROM (
                    SELECT media_item_id, embedding <=> $3::vector as distance
                    FROM visual_analysis
                    WHERE user_id = $2
                      AND deleted = false
                    ORDER BY embedding <=> $3::vector
                    LIMIT $4 * 4
                ) sub_ordered
                ORDER BY media_item_id, distance
            ) sub_unique
            ORDER BY distance
            LIMIT $4
        ),
        merged AS (
            SELECT id, rank, 1 as is_fts, 0 as is_vec FROM fts
            UNION ALL
            SELECT id, rank, 0 as is_fts, 1 as is_vec FROM vec
        ),
        scored_candidates AS (
            SELECT
                id,
                SUM(
                    CASE
                        WHEN is_fts = 1 THEN $7::float8 / ($6::float8 + rank::float8)
                        WHEN is_vec = 1 THEN $8::float8 / ($6::float8 + rank::float8)
                        ELSE 0
                    END
                )::real as combined_score
            FROM merged
            GROUP BY id
        )
        SELECT
            mi.id::text as "id!",
            mi.is_video as "is_video!",
            mi.has_thumbnails as "has_thumbnails!",
            mi.duration_ms as "duration_ms: i32",
            (mi.width::real / mi.height::real) as "ratio!"
        FROM scored_candidates sc
        JOIN media_item mi ON mi.id = sc.id
        ORDER BY sc.combined_score DESC
        LIMIT $5
         "#,
        query,                 // $1
        user.id,               // $2
        vector_param as _,     // $3
        candidate_limit,       // $4
        limit,                 // $5
        k,                     // $6
        config.text_weight,    // $7
        config.semantic_weight // $8
    )
        .fetch_all(pool)
        .await?;

    Ok(items)
}

pub async fn get_search_suggestions(
    user: &User,
    pool: &PgPool,
    embedder: Arc<TextEmbedder>,
    query: &str,
    config: SearchMediaConfig,
) -> Result<SearchSuggestionsResponse, SearchError> {
    let suggestions_limit = config.limit.unwrap_or(10).min(50);
    let ilike_query = format!("%{}%", query);

    let suggestions = sqlx::query_scalar!(
        r#"
        WITH user_suggestions AS (
            -- LLM terms
            SELECT DISTINCT search_term as suggestion
            FROM classification c
            JOIN visual_analysis va ON c.visual_analysis_id = va.id
            WHERE va.user_id = $1 AND search_term ILIKE $2
            UNION
            -- People
            SELECT name as suggestion
            FROM person
            WHERE user_id = $1 AND name ILIKE $2
            UNION
            -- Locations
            SELECT DISTINCT l.name as suggestion
            FROM location l
            JOIN gps g ON l.id = g.location_id
            JOIN media_item mi ON g.media_item_id = mi.id
            WHERE mi.user_id = $1 AND l.name ILIKE $2
            UNION
            SELECT DISTINCT l.admin1 as suggestion
            FROM location l
            JOIN gps g ON l.id = g.location_id
            JOIN media_item mi ON g.media_item_id = mi.id
            WHERE mi.user_id = $1 AND l.admin1 ILIKE $2
            UNION
            SELECT DISTINCT l.admin2 as suggestion
            FROM location l
            JOIN gps g ON l.id = g.location_id
            JOIN media_item mi ON g.media_item_id = mi.id
            WHERE mi.user_id = $1 AND l.admin2 ILIKE $2
            UNION
            SELECT DISTINCT l.country_name as suggestion
            FROM location l
            JOIN gps g ON l.id = g.location_id
            JOIN media_item mi ON g.media_item_id = mi.id
            WHERE mi.user_id = $1 AND l.country_name ILIKE $2
        )
        SELECT suggestion FROM user_suggestions
        WHERE suggestion IS NOT NULL AND suggestion != ''
        ORDER BY suggestion
        LIMIT $3
        "#,
        user.id,
        ilike_query,
        suggestions_limit
    )
    .fetch_all(pool)
    .await?;

    let items = search_media(user, pool, embedder, query, config).await?;

    Ok(SearchSuggestionsResponse {
        suggestions: suggestions.into_iter().flatten().collect(),
        items,
    })
}
