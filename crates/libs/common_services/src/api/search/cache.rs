use crate::api::search::error::SearchError;
use moka::future::Cache;
use open_clip_inference::TextEmbedder;
use pgvector::Vector;
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};

static TEXT_EMBEDDING_CACHE: OnceLock<Cache<(String, String), Vec<f32>>> = OnceLock::new();

fn get_cache() -> Cache<(String, String), Vec<f32>> {
    TEXT_EMBEDDING_CACHE
        .get_or_init(|| Cache::builder().max_capacity(10_000).build())
        .clone()
}

pub async fn get_cached_text_embedding(
    query: &str,
    model_id: &str,
    pool: &PgPool,
    embedder: Arc<TextEmbedder>,
) -> Result<Vec<f32>, SearchError> {
    let cache = get_cache();
    let key = (model_id.to_string(), query.to_string());

    // In memory cache hit
    if let Some(emb) = cache.get(&key).await {
        return Ok(emb);
    }

    let db_result = sqlx::query!(
        r#"
        SELECT embedding::vector as "embedding!: Vector"
        FROM text_embedding_cache
        WHERE model_id = $1 AND text = $2
        "#,
        model_id,
        query
    )
    .fetch_optional(pool)
    .await?;

    // DB cache hit
    if let Some(row) = db_result {
        let emb = row.embedding.to_vec();
        cache.insert(key, emb.clone()).await;
        return Ok(emb);
    }

    // Calculate embedding
    let query_str = query.to_string();
    let compute_emb = tokio::task::spawn_blocking(move || embedder.embed_text(&query_str))
        .await??
        .to_vec();

    let vector_param = Vector::from(compute_emb.clone());
    let pool_clone = pool.clone();
    let model_id_clone = model_id.to_string();
    let query_clone = query.to_string();

    tokio::spawn(async move {
        let _ = sqlx::query!(
            r#"
            INSERT INTO text_embedding_cache (model_id, text, embedding)
            VALUES ($1, $2, $3::vector)
            ON CONFLICT (model_id, text) DO NOTHING
            "#,
            model_id_clone,
            query_clone,
            vector_param as _
        )
        .execute(&pool_clone)
        .await;
    });

    cache.insert(key, compute_emb.clone()).await;

    Ok(compute_emb)
}
