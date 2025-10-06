use crate::utils::get_relative_path_str;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

pub async fn enqueue_file_ingest(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path_str = get_relative_path_str(file)?;
    let mut tx = pool.begin().await?;

    let existing_job: Option<i32> = sqlx::query_scalar!(
        "SELECT id FROM job_queue WHERE relative_path = $1 AND job_type = 'INGEST'",
        relative_path_str
    )
    .fetch_optional(&mut *tx)
    .await?;
    if existing_job.is_some() {
        info!(
            "Tried to enqueue job that already existed for file {:?}",
            file
        );
        return Ok(());
    }
    let is_failure: Option<i32> = sqlx::query_scalar!(
        r#"
            SELECT id
            FROM queue_failures
            WHERE relative_path = $1 AND job_type = 'INGEST'
        "#,
        relative_path_str,
    )
    .fetch_optional(&mut *tx)
    .await?;
    if is_failure.is_some() {
        info!(
            "Tried to enqueue job for file that failed before, file: {:?}",
            file
        );
        return Ok(());
    }

    info!("Enqueueing file creation: {:?}", file.display());
    sqlx::query!(
        "
        INSERT INTO job_queue (relative_path, job_type, priority)
        VALUES ($1, 'INGEST', 10)
        ON CONFLICT (relative_path) DO UPDATE
        SET
            job_type = 'INGEST',
            priority = 10,
            retry_count = 0,
            created_at = NOW()
        ",
        relative_path_str,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

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
