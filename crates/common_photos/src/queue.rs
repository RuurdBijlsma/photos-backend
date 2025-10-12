use crate::settings;
use crate::utils::{relative_path_no_exist, user_id_from_username, username_from_path};
use color_eyre::eyre::eyre;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

// Priorities, lower is handled sooner
const REMOVE_FILE_PRIORITY: i32 = 0;
const INGEST_PHOTO_PRIORITY: i32 = 10;
const INGEST_VIDEO_PRIORITY: i32 = 20;
const ANALYSIS_PRIORITY: i32 = 30;

/// Enqueues an 'INGEST' job for a given file.
///
/// # Errors
///
/// This function will return an error if:
/// - The relative path for the file cannot be determined.
/// - A username cannot be extracted from the file path.
/// - The extracted username does not correspond to an existing user in the database.
/// - The database query to insert the job fails.
pub async fn enqueue_file_ingest(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path_str = relative_path_no_exist(file)?;
    let Some(username) = username_from_path(file) else {
        return Err(eyre!("Failed to get username from path: {:?}.", file));
    };
    let Some(user_id) = user_id_from_username(&username, pool).await? else {
        return Err(eyre!("User '{}' does not exist in db.", username));
    };

    let video_extensions = &settings().thumbnail_generation.video_extensions;
    let priority = file
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .map_or(INGEST_VIDEO_PRIORITY, |ext| {
            if video_extensions.contains(&ext) {
                INGEST_VIDEO_PRIORITY
            } else {
                INGEST_PHOTO_PRIORITY
            }
        });

    let rows_affected = sqlx::query!(
        r#"
        INSERT INTO job_queue (relative_path, user_id, job_type, priority)
        SELECT $1, $2, 'INGEST', $3
        WHERE NOT EXISTS (
            SELECT 1 FROM queue_failures WHERE relative_path = $1 AND job_type = 'INGEST'
        )
        ON CONFLICT (relative_path, job_type) DO NOTHING
        "#,
        relative_path_str,
        user_id,
        priority
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected > 0 {
        info!("Enqueued 'INGEST' job for: {:?}", relative_path_str);
    } else {
        info!(
            "Skipped enqueue for {:?}: 'INGEST' job already exists or has failed.",
            relative_path_str
        );
    }

    Ok(())
}

/// Enqueues an `ANALYSIS` job for a given file.
///
/// # Errors
///
/// This function will return an error if the database query to insert the job fails.
pub async fn enqueue_analysis(
    relative_path: &str,
    user_id: i32,
    pool: &PgPool,
) -> color_eyre::Result<()> {
    let rows_affected = sqlx::query!(
        r#"
        INSERT INTO job_queue (relative_path, user_id, job_type, priority)
        SELECT $1, $2, 'ANALYSIS', $3
        WHERE NOT EXISTS (
            SELECT 1 FROM queue_failures WHERE relative_path = $1 AND job_type = 'ANALYSIS'
        )
        ON CONFLICT (relative_path, job_type) DO UPDATE
        SET
            priority = $3,
            retry_count = 0,
            created_at = NOW()
        "#,
        relative_path,
        user_id,
        ANALYSIS_PRIORITY
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected > 0 {
        info!("Enqueued 'ANALYSIS' job for: {:?}", relative_path);
    } else {
        info!(
            "Skipped enqueue for {:?}: 'ANALYSIS' job already exists or has failed.",
            relative_path
        );
    }

    Ok(())
}

/// Enqueues a 'REMOVE' job for a given file.
/// This will remove any existing 'INGEST' or `ANALYSIS` jobs for the same file.
///
/// # Errors
///
/// This function will return an error if:
/// - The relative path for the file cannot be determined.
/// - The database transaction fails to begin, execute, or commit. This could be due to
///   issues with deleting existing jobs or inserting the new 'REMOVE' job.
pub async fn enqueue_file_remove(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path = relative_path_no_exist(file)?;
    let Some(username) = username_from_path(file) else {
        return Err(eyre!("Failed to get username from path: {:?}.", file));
    };
    let Some(user_id) = user_id_from_username(&username, pool).await? else {
        return Err(eyre!("User '{}' does not exist in db.", username));
    };
    let mut tx = pool.begin().await?;

    info!(
        "Enqueueing file removal, replacing all other jobs for: {:?}",
        file
    );

    // 1. Delete all existing jobs for this path. (can hang because of lock on relative_path)
    sqlx::query!(
        "DELETE FROM job_queue WHERE relative_path = $1",
        relative_path
    )
    .execute(&mut *tx)
    .await?;

    // 2. Insert the new 'REMOVE' job.
    // ON CONFLICT is used here as a safeguard in case two 'REMOVE' jobs are enqueued concurrently.
    sqlx::query!(
        "
        INSERT INTO job_queue (relative_path, user_id, job_type, priority)
        VALUES ($1, $2, 'REMOVE', $3)
        ON CONFLICT (relative_path, job_type) DO UPDATE
        SET
          priority = $3,
          retry_count = 0,
          created_at = NOW();
        ",
        relative_path,
        user_id,
        REMOVE_FILE_PRIORITY
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
