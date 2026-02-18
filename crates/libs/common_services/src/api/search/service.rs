use crate::api::search::error::SearchError;
use crate::api::search::interfaces::SearchResultItem;
use crate::database::app_user::User;
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
) -> Result<Vec<SearchResultItem>, SearchError> {
    let query_str = query.to_string();
    let embedder = embedder.clone();
    let query_embedding = tokio::task::spawn_blocking(move || embedder.embed_text(&query_str))
        .await??
        .to_vec();
    let vector_param = Vector::from(query_embedding);

    let limit = config.limit.unwrap_or(100).min(500);
    let candidate_limit = limit * 3 + 300;
    let k = 60.0f64;

    let items = sqlx::query_as!(SearchResultItem,
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
                media_item_id as id,
                1 - (embedding <=> $3::vector) as score,
                ROW_NUMBER() OVER (ORDER BY embedding <=> $3::vector) as rank
            FROM visual_analysis
            WHERE user_id = $2
              AND deleted = false
            ORDER BY embedding <=> $3::vector
            LIMIT $4
        ),
        merged AS (
            SELECT id, score, rank, 1 as is_fts, 0 as is_vec FROM fts
            UNION ALL
            SELECT id, score, rank, 0 as is_fts, 1 as is_vec FROM vec
        ),
        scored_candidates AS (
            SELECT
                id,
                MAX(score) FILTER (WHERE is_fts = 1) as fts_score,
                MAX(score) FILTER (WHERE is_vec = 1) as vector_score,
                MAX(rank) FILTER (WHERE is_fts = 1) as fts_rank,
                MAX(rank) FILTER (WHERE is_vec = 1) as vector_rank,
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
            mi.id,
            mi.is_video,
            mi.has_thumbnails,
            mi.duration_ms,
            mi.taken_at_local,
            (mi.width::real / mi.height::real) as "ratio!",
            coalesce(sc.fts_score, 0)::real as "fts_score!",
            coalesce(sc.vector_score, 0)::real as "vector_score!",
            sc.fts_rank::integer as "fts_rank",
            sc.vector_rank::integer as "vector_rank",
            sc.combined_score as "combined_score!"
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

pub async fn get_search_suggestions(_user: &User, _pool: &PgPool) -> Result<String, SearchError> {
    Ok("asdf".to_owned())
}
