use crate::api::app_error::AppError;
use crate::api::photos::removal::delete_item_and_thumbnails;
use common_types::pb::api::{OrderedMediaResponse, SimpleTimelineItem};
use sqlx::PgPool;
use std::path::Path;
use tokio::fs;
use tracing::info;

/// Fetches all soft-deleted media items for the user, sorted by sort_timestamp descending.
pub async fn get_trash_items(
    pool: &PgPool,
    user_id: i32,
) -> Result<OrderedMediaResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            is_video,
            has_thumbnails,
            duration_ms::INT AS duration_ms,
            (width::real / height::real) AS "ratio!"
        FROM media_item
        WHERE user_id = $1 AND deleted = true
        ORDER BY sort_timestamp DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| SimpleTimelineItem {
            id: r.id,
            is_video: r.is_video,
            has_thumbnails: r.has_thumbnails,
            duration_ms: r.duration_ms,
            ratio: r.ratio,
        })
        .collect();

    Ok(OrderedMediaResponse { items })
}

/// Soft-deletes a list of media items.
pub async fn soft_delete_items(
    pool: &PgPool,
    user_id: i32,
    ids: &[String],
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        WITH updated_items AS (
            UPDATE media_item
            SET deleted = true
            WHERE id = ANY($1) AND user_id = $2
            RETURNING id
        )
        UPDATE visual_analysis
        SET deleted = true
        FROM updated_items
        WHERE visual_analysis.media_item_id = updated_items.id
        "#,
        ids,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Restores a list of soft-deleted media items.
pub async fn restore_items(pool: &PgPool, user_id: i32, ids: &[String]) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        WITH updated_items AS (
            UPDATE media_item
            SET deleted = false
            WHERE id = ANY($1) AND user_id = $2
            RETURNING id
        )
        UPDATE visual_analysis
        SET deleted = false
        FROM updated_items
        WHERE visual_analysis.media_item_id = updated_items.id
        "#,
        ids,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Permanently deletes media items from the database and filesystem.
pub async fn perma_delete_items(
    pool: &PgPool,
    user_id: i32,
    ids: &[String],
    media_root: &Path,
    thumbnail_root: &Path,
) -> Result<(), AppError> {
    // 1. Fetch relative paths of items belonging to the user
    let items = sqlx::query!(
        r#"
        SELECT relative_path
        FROM media_item
        WHERE id = ANY($1) AND user_id = $2
        "#,
        ids,
        user_id
    )
    .fetch_all(pool)
    .await?;

    // 2. Delete original files from disk and then clean up DB and thumbnails
    for item in items {
        let file_path = media_root.join(&item.relative_path);
        if file_path.exists() {
            info!("Removing original media file: {}", file_path.display());
            if let Err(e) = fs::remove_file(&file_path).await {
                // Log and continue or return error? Let's return error if we fail to remove,
                // but if the file is already missing, delete_item_and_thumbnails will still clean up the DB.
                // We check exists() first, so if it's there and we fail to delete, it's a real error.
                return Err(AppError::Internal(color_eyre::eyre::eyre!(
                    "Failed to delete original file {}: {}",
                    item.relative_path,
                    e
                )));
            }
        }
        // Clean up database records and thumbnails folder.
        delete_item_and_thumbnails(pool, thumbnail_root, &item.relative_path).await?;
    }

    Ok(())
}
