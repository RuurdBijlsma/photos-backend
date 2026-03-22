use crate::api::search::error::SearchError;
use crate::database::app_user::User;
use common_types::pb::api::{
    SearchSuggestion, SearchSuggestionsResponse, SimpleTimelineItem, SuggestionType,
};
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, Copy)]
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
    query: &str,
    limit: Option<i64>,
) -> Result<SearchSuggestionsResponse, SearchError> {
    if query.trim().is_empty() {
        return Ok(SearchSuggestionsResponse::default());
    }

    let limit = limit.unwrap_or(10).min(50);
    let ilike_query = format!("%{query}%");
    let suggestions = sqlx::query!(
        r#"
        WITH matched_terms AS (
            (SELECT c.search_term as suggestion, COUNT(DISTINCT va.media_item_id) as photo_count, 'SEARCH' as "type!", NULL as "id"
            FROM classification c
            JOIN visual_analysis va ON c.visual_analysis_id = va.id
            WHERE va.user_id = $1
              AND c.search_term ILIKE $2
              AND c.search_term != ''
            GROUP BY c.search_term
            LIMIT $3 * 2)

            UNION ALL

            (SELECT p.name as suggestion, COUNT(DISTINCT va.media_item_id) as photo_count, 'SEARCH' as "type!", NULL as "id"
            FROM person p
            JOIN face f ON f.person_id = p.id
            JOIN visual_analysis va ON f.visual_analysis_id = va.id
            WHERE p.user_id = $1
              AND p.name ILIKE $2
              AND p.name != ''
            GROUP BY p.name
            LIMIT $3 * 2)

            UNION ALL

            (SELECT loc.val as suggestion, COUNT(DISTINCT g.media_item_id) as photo_count, 'SEARCH' as "type!", NULL as "id"
            FROM (
                SELECT id, name as val FROM location WHERE name ILIKE $2
                UNION
                SELECT id, admin1 as val FROM location WHERE admin1 ILIKE $2
                UNION
                SELECT id, country_name as val FROM location WHERE country_name ILIKE $2
            ) loc
            JOIN gps g ON g.location_id = loc.id
            JOIN media_item mi ON g.media_item_id = mi.id
            WHERE mi.user_id = $1 AND mi.deleted = false
            GROUP BY loc.val
            LIMIT $3 * 2)

            UNION ALL

            (SELECT a.name as suggestion, COUNT(DISTINCT am.media_item_id) as photo_count, 'ALBUM' as "type!", a.id::text as "id"
            FROM album a
            LEFT JOIN album_media_item am ON a.id = am.album_id
            LEFT JOIN album_collaborator ac ON a.id = ac.album_id AND ac.user_id = $1
            WHERE (a.owner_id = $1 OR ac.user_id IS NOT NULL)
              AND a.name ILIKE $2
              AND a.name != ''
            GROUP BY a.name, a.id
            LIMIT $3 * 2)
        )
        SELECT suggestion as "suggestion!", "type!" as "type!", "id" as "id?", SUM(photo_count)::int8 as "photo_count!"
        FROM matched_terms
        GROUP BY suggestion, "type!", "id"
        ORDER BY (CASE WHEN "type!" = 'ALBUM' THEN 0 ELSE 1 END), "photo_count!" DESC, suggestion ASC
        LIMIT $3
        "#,
        user.id,
        ilike_query,
        limit as i32
    )
        .fetch_all(pool)
        .await?;

    Ok(SearchSuggestionsResponse {
        suggestions: suggestions
            .into_iter()
            .map(|row| SearchSuggestion {
                text: row.suggestion,
                suggestion_type: match row.r#type.as_str() {
                    "ALBUM" => SuggestionType::Album as i32,
                    _ => SuggestionType::Search as i32,
                },
                id: row.id,
            })
            .collect(),
    })
}

pub async fn get_random_search_suggestion(
    user: &User,
    pool: &PgPool,
) -> Result<Option<String>, SearchError> {
    let row = sqlx::query!(
        r#"
        WITH matched_terms AS (
            (SELECT c.search_term as suggestion
            FROM classification c
            JOIN visual_analysis va ON c.visual_analysis_id = va.id
            WHERE va.user_id = $1
              AND c.search_term != ''
            GROUP BY c.search_term
            ORDER BY COUNT(DISTINCT va.media_item_id) DESC
            LIMIT 100)

            UNION ALL

            (SELECT p.name as suggestion
            FROM person p
            JOIN face f ON f.person_id = p.id
            JOIN visual_analysis va ON f.visual_analysis_id = va.id
            WHERE p.user_id = $1
              AND p.name != ''
            GROUP BY p.name
            ORDER BY COUNT(DISTINCT va.media_item_id) DESC
            LIMIT 100)

            UNION ALL

            (SELECT val as suggestion
            FROM (
                (SELECT l.name as val, COUNT(mi.id) as cnt
                FROM location l
                JOIN gps g ON g.location_id = l.id
                JOIN media_item mi ON g.media_item_id = mi.id
                WHERE mi.user_id = $1 AND mi.deleted = false AND l.name != ''
                GROUP BY l.name
                LIMIT 100)
                UNION ALL
                (SELECT l.admin1 as val, COUNT(mi.id) as cnt
                FROM location l
                JOIN gps g ON g.location_id = l.id
                JOIN media_item mi ON g.media_item_id = mi.id
                WHERE mi.user_id = $1 AND mi.deleted = false AND l.admin1 != ''
                GROUP BY l.admin1
                LIMIT 100)
                UNION ALL
                (SELECT l.country_name as val, COUNT(mi.id) as cnt
                FROM location l
                JOIN gps g ON g.location_id = l.id
                JOIN media_item mi ON g.media_item_id = mi.id
                WHERE mi.user_id = $1 AND mi.deleted = false AND l.country_name != ''
                GROUP BY l.country_name
                LIMIT 100)
            ) locs
            ORDER BY cnt DESC
            LIMIT 100)

            UNION ALL

            (SELECT a.name as suggestion
            FROM album a
            LEFT JOIN album_collaborator ac ON a.id = ac.album_id AND ac.user_id = $1
            JOIN album_media_item am ON a.id = am.album_id
            WHERE (a.owner_id = $1 OR ac.user_id IS NOT NULL)
              AND a.name != ''
            GROUP BY a.name, a.id
            ORDER BY COUNT(DISTINCT am.media_item_id) DESC
            LIMIT 100)
        )
        SELECT suggestion as "suggestion!"
        FROM matched_terms
        ORDER BY RANDOM()
        LIMIT 1
        "#,
        user.id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.suggestion))
}
