use crate::utils::get_relative_path_str;
use sqlx::PgPool;
use std::path::Path;

pub async fn enqueue_file_ingest(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path_str = get_relative_path_str(file)?;
    let mut tx = pool.begin().await?;

    println!("Enqueueing file creation: {:?}", file);
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

    println!("Enqueueing file deletion: {:?}", file);
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
