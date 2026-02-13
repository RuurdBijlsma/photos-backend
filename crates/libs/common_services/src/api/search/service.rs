use crate::api::search::error::SearchError;
use crate::api::search::interfaces::SearchResultItem;
use crate::database::app_user::User;
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;

pub struct SearchMediaConfig {
    pub limit: Option<i64>,
    pub threshold: Option<f64>,
    pub semantic_weight: f64,
    pub text_weight: f32,
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

    let limit = config.limit.unwrap_or(50).min(100);
    let cutoff = config.threshold.unwrap_or(0.0);
    let candidate_limit = limit * 3;

    // --- TEMPORARY EXPLAIN ANALYZE BLOCK ---
    let explain_query = r"
        EXPLAIN (ANALYZE, COSTS, VERBOSE, BUFFERS)
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
            WHERE user_id = $2
            ORDER BY embedding <=> $3::vector
            LIMIT $4
        )
        SELECT
            mi.id,
            mi.is_video,
            mi.has_thumbnails,
            mi.duration_ms,
            mi.taken_at_local,
            (mi.width::real / mi.height::real) as ratio,
            coalesce(f.rank, 0)::real as fts_score,
            coalesce(v.similarity, 0)::real as vector_score,
            (coalesce(f.rank, 0) * $8 + coalesce(v.similarity, 0) * $7)::real as combined_score
        FROM media_item mi
        LEFT JOIN fts_search f ON mi.id = f.id
        LEFT JOIN vector_search v ON mi.id = v.id
        WHERE (f.id IS NOT NULL OR v.id IS NOT NULL)
          AND (coalesce(f.rank, 0) * $8 + coalesce(v.similarity, 0) * $7) >= $6
        ORDER BY mi.sort_timestamp DESC
        LIMIT $5
    ";

    let rows = sqlx::query(explain_query)
        .bind(query)
        .bind(user.id)
        .bind(vector_param.clone())
        .bind(candidate_limit)
        .bind(limit)
        .bind(cutoff)
        .bind(config.semantic_weight)
        .bind(config.text_weight)
        .fetch_all(pool)
        .await?;

    println!("\n--- EXPLAIN ANALYZE OUTPUT ---");
    for row in rows {
        let line: String = sqlx::Row::get(&row, 0);
        println!("{line}");
    }
    println!("------------------------------\n");
    // --- END TEMPORARY BLOCK ---

    let items = sqlx::query_as!(
        SearchResultItem,
        r#"
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
            WHERE user_id = $2
            ORDER BY embedding <=> $3::vector
            LIMIT $4
        )
        SELECT
            mi.id,
            mi.is_video,
            mi.has_thumbnails,
            mi.duration_ms,
            mi.taken_at_local,
            (mi.width::real / mi.height::real) as "ratio!",
            coalesce(f.rank, 0)::real as "fts_score!",
            coalesce(v.similarity, 0)::real as "vector_score!",
            (coalesce(f.rank, 0) * $8 + coalesce(v.similarity, 0) * $7)::real as "combined_score!"
        FROM media_item mi
        LEFT JOIN fts_search f ON mi.id = f.id
        LEFT JOIN vector_search v ON mi.id = v.id
        WHERE (f.id IS NOT NULL OR v.id IS NOT NULL)
          AND (coalesce(f.rank, 0) * $8 + coalesce(v.similarity, 0) * $7) >= $6
        ORDER BY mi.sort_timestamp DESC
        LIMIT $5
        "#,
        query,
        user.id,
        vector_param as _,
        candidate_limit,
        limit,
        cutoff,
        config.semantic_weight,
        config.text_weight
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}
