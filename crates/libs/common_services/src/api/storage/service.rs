use crate::api::app_error::AppError;
use chrono::{DateTime, Utc};
use common_types::pb::api::{StorageReviewItem, StorageReviewResponse, StorageSummaryResponse};
use sqlx::{PgPool, Row};

const REVIEW_LIMIT: i64 = 250;
const REVIEW_MIN_SIZE_BYTES: i64 = 10 * 1024 * 1024;
const BLURRY_QUALITY_THRESHOLD: f64 = 50.0;

pub async fn get_storage_summary(
    pool: &PgPool,
    user_id: i32,
) -> Result<StorageSummaryResponse, AppError> {
    let large_row = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(size_bytes), 0)::BIGINT AS "large_potential_savings!",
            COUNT(*)::INT AS "large_item_count!"
        FROM (
            SELECT mf.size_bytes
            FROM media_item mi
            JOIN media_features mf ON mf.media_item_id = mi.id
            WHERE mi.user_id = $1
              AND mi.deleted = false
              AND mf.size_bytes > $2
            ORDER BY mf.size_bytes DESC
            LIMIT $3
        ) large_items
        "#,
        user_id,
        REVIEW_MIN_SIZE_BYTES,
        REVIEW_LIMIT,
    )
        .fetch_one(pool)
        .await?;

    let blurry_row = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(size_bytes), 0)::BIGINT AS "blurry_potential_savings!",
            COUNT(*)::INT AS "blurry_item_count!"
        FROM (
            SELECT DISTINCT mi.id, mf.size_bytes
            FROM media_item mi
            JOIN media_features mf ON mf.media_item_id = mi.id
            JOIN visual_analysis va ON va.media_item_id = mi.id
            JOIN measured_quality mq ON mq.visual_analysis_id = va.id
            WHERE mi.user_id = $1
              AND mi.deleted = false
              AND mi.is_video = false
              AND mq.weighted_score < $2
        ) blurry_items
        "#,
        user_id,
        BLURRY_QUALITY_THRESHOLD,
    )
        .fetch_one(pool)
        .await?;

    Ok(StorageSummaryResponse {
        large_potential_savings: large_row.large_potential_savings,
        large_item_count: large_row.large_item_count,
        blurry_potential_savings: blurry_row.blurry_potential_savings,
        blurry_item_count: blurry_row.blurry_item_count,
    })
}

pub async fn get_large_storage_items(
    pool: &PgPool,
    user_id: i32,
) -> Result<StorageReviewResponse, AppError> {
    get_review_items(
        pool,
        user_id,
        r#"
        SELECT
            mi.id,
            mi.is_video,
            mi.has_thumbnails,
            mi.duration_ms::INT AS duration_ms,
            (mi.width::REAL / mi.height::REAL) AS ratio,
            mf.size_bytes,
            mi.sort_timestamp,
            NULL::REAL AS weighted_score
        FROM media_item mi
        JOIN media_features mf ON mf.media_item_id = mi.id
        WHERE mi.user_id = $1
          AND mi.deleted = false
          AND mf.size_bytes > $2
        ORDER BY mf.size_bytes DESC
        LIMIT $3
        "#,
        Some(REVIEW_MIN_SIZE_BYTES),
        Some(REVIEW_LIMIT),
    )
    .await
}

pub async fn get_blurry_storage_items(
    pool: &PgPool,
    user_id: i32,
) -> Result<StorageReviewResponse, AppError> {
    get_review_items(
        pool,
        user_id,
        r#"
        SELECT *
        FROM (
            SELECT DISTINCT ON (mi.id)
                mi.id,
                mi.is_video,
                mi.has_thumbnails,
                mi.duration_ms::INT AS duration_ms,
                (mi.width::REAL / mi.height::REAL) AS ratio,
                mf.size_bytes,
                mi.sort_timestamp,
                mq.weighted_score::REAL AS weighted_score
            FROM media_item mi
            JOIN media_features mf ON mf.media_item_id = mi.id
            JOIN visual_analysis va ON va.media_item_id = mi.id
            JOIN measured_quality mq ON mq.visual_analysis_id = va.id
            WHERE mi.user_id = $1
              AND mi.deleted = false
              AND mi.is_video = false
              AND mq.weighted_score < $2
            ORDER BY mi.id, mq.weighted_score ASC
        ) items
        ORDER BY size_bytes DESC
        "#,
        Some(BLURRY_QUALITY_THRESHOLD as i64),
        None,
    )
    .await
}

async fn get_review_items(
    pool: &PgPool,
    user_id: i32,
    sql: &str,
    threshold: Option<i64>,
    limit: Option<i64>,
) -> Result<StorageReviewResponse, AppError> {
    let mut query = sqlx::query(sql).bind(user_id);
    if let Some(threshold) = threshold {
        query = query.bind(threshold);
    }
    if let Some(limit) = limit {
        query = query.bind(limit);
    }

    let rows = query.fetch_all(pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    let mut total_size = 0_i64;

    for row in rows {
        let file_size = row.get::<i64, _>("size_bytes");
        total_size += file_size;
        let timestamp = row.get::<DateTime<Utc>, _>("sort_timestamp").to_rfc3339();

        items.push(StorageReviewItem {
            id: row.get("id"),
            is_video: row.get("is_video"),
            has_thumbnails: row.get("has_thumbnails"),
            duration_ms: row.get("duration_ms"),
            ratio: row.get("ratio"),
            file_size,
            timestamp,
            weighted_score: row.get("weighted_score"),
        });
    }

    Ok(StorageReviewResponse { items, total_size })
}
