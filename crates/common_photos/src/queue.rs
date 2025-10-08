use crate::get_thumbnail_options;
use crate::utils::get_relative_path_str;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

/// Enqueues an 'INGEST' job for a given file.
/// If an 'INGEST' job for the file already exists or has previously failed, it will not be re-enqueued.
///
/// # Errors
///
/// * `color_eyre::Result` propagated from `get_relative_path_str`.
/// * `sqlx::Error` for database-related issues, including transaction management.
pub async fn enqueue_file_ingest(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path_str = get_relative_path_str(file)?;

    let config = get_thumbnail_options();
    let video_extensions = config.video_extensions.clone();
    let priority = file
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .map_or(20, |ext| {
            // Lower priority for videos
            if video_extensions.contains(&ext) {
                20
            } else {
                10
            }
        });

    let rows_affected = sqlx::query!(
        r#"
        INSERT INTO job_queue (relative_path, job_type, priority)
        SELECT $1, 'INGEST', $2
        WHERE NOT EXISTS (
            SELECT 1 FROM queue_failures WHERE relative_path = $1 AND job_type = 'INGEST'
        )
        ON CONFLICT (relative_path) DO UPDATE
        SET
            job_type = 'INGEST',
            priority = $2,
            retry_count = 0,
            created_at = NOW()
        WHERE
            -- Only update if it's not already an INGEST job, preventing redundant work.
            job_queue.job_type IS DISTINCT FROM 'INGEST'
        "#,
        relative_path_str,
        priority
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected > 0 {
        info!("Enqueued job for: {:?}", relative_path_str);
    } else {
        info!(
            "Skipped enqueue for {:?}: job already exists or has failed.",
            file
        );
    }

    Ok(())
}

/// Enqueues a 'REMOVE' job for a given file.
/// If a job for the file already exists, it will be updated to a 'REMOVE' job.
///
/// # Errors
///
/// * `color_eyre::Result` propagated from `get_relative_path_str`.
/// * `sqlx::Error` for database-related issues, including transaction management.
pub async fn enqueue_file_remove(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path = get_relative_path_str(file)?;
    let mut tx = pool.begin().await?;

    info!("Enqueueing file removal: {:?}", file);
    sqlx::query!(
        "
        INSERT INTO job_queue (relative_path, job_type, priority)
        VALUES ($1, 'REMOVE', 0)
        ON CONFLICT (relative_path) DO UPDATE
        SET
          job_type = 'REMOVE',
          priority = 0,
          retry_count = 0,
          created_at = NOW();
        ",
        relative_path
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
