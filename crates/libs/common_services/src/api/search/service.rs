use crate::api::search::error::SearchError;
use crate::api::search::interfaces::SearchResultItem;
use crate::database::app_user::User;
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;

pub struct SearchMediaConfig {
    pub limit: Option<i64>,
    pub semantic_weight: f64,
    pub text_weight: f64,
}

pub async fn search_media(
    user: &User,
    pool: &PgPool,
    embedder: &TextEmbedder,
    query: &str,
    config: SearchMediaConfig,
) -> Result<Vec<SearchResultItem>, SearchError> {
    let query_embedding = embedder.embed_text(query)?.to_vec();
    let vector_param = Vector::from(query_embedding);

    let limit = config.limit.unwrap_or(50).min(500);
    let candidate_limit = limit * 5;

    // RRF constant 'k' (standard is 60)
    let k = 60.0f64;

    let items = sqlx::query_as!(
        SearchResultItem,
        r#"
        WITH fts_search AS (
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
        vector_search AS (
            SELECT
                media_item_id as id,
                1 - (embedding <=> $3::vector) as score,
                ROW_NUMBER() OVER (ORDER BY embedding <=> $3::vector) as rank
            FROM visual_analysis
            WHERE user_id = $2
            LIMIT $4
        )
        SELECT
            mi.id,
            mi.is_video,
            mi.has_thumbnails,
            mi.duration_ms,
            mi.taken_at_local,
            (mi.width::real / mi.height::real) as "ratio!",
            coalesce(f.score, 0)::real as "fts_score!",
            coalesce(v.score, 0)::real as "vector_score!",
            f.rank::integer as "fts_rank",
            v.rank::integer as "vector_rank",
            (
                coalesce($7::float8 / ($6::float8 + f.rank::float8), 0.0) +
                coalesce($8::float8 / ($6::float8 + v.rank::float8), 0.0)
            )::real as "combined_score!"
        FROM media_item mi
        LEFT JOIN fts_search f ON mi.id = f.id
        LEFT JOIN vector_search v ON mi.id = v.id
        WHERE (f.id IS NOT NULL OR v.id IS NOT NULL)
          AND mi.deleted = false
        ORDER BY "combined_score!" DESC
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
