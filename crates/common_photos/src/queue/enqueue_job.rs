use crate::user_id_from_relative_path;
use crate::{is_video_file, media_dir, JobType};
use color_eyre::eyre::Result;
use sqlx::postgres::PgQueryResult;
use sqlx::{PgConnection, PgPool};

pub async fn enqueue_ingest_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let is_video = is_video_file(&media_dir().join(relative_path));
    let priority = if is_video { 20 } else { 10 };

    let mut tx = pool.begin().await?;
    enqueue_job(&mut tx, relative_path, JobType::Ingest, priority).await?;
    tx.commit().await?;

    Ok(())
}

pub async fn enqueue_analysis_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let mut tx = pool.begin().await?;
    enqueue_job(&mut tx, relative_path, JobType::Analysis, 100).await?;
    tx.commit().await?;

    Ok(())
}

pub async fn enqueue_remove_job(pool: &PgPool, relative_path: &str) -> Result<()> {
    let mut tx = pool.begin().await?;

    // cancel queued ingest/analysis for same file
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'cancelled'
        WHERE relative_path = $1
          AND status = 'queued'
          AND job_type IN ('ingest', 'analysis')
        "#,
        relative_path
    )
    .execute(&mut *tx)
    .await?;

    // enqueue remove with the highest priority
    enqueue_job(&mut tx, relative_path, JobType::Remove, 0).await?;

    tx.commit().await?;
    Ok(())
}

async fn enqueue_job(
    tx: &mut PgConnection,
    relative_path: &str,
    job_type: JobType,
    priority: i32,
) -> Result<PgQueryResult> {
    // todo: probably don't enqueue job if it's marked as failed?
    let user_id = user_id_from_relative_path(relative_path, &mut *tx).await?;

    sqlx::query!(
        r#"
        INSERT INTO jobs (relative_path, job_type, priority, user_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#,
        relative_path,
        job_type as JobType,
        priority,
        user_id
    )
    .execute(tx)
    .await
    .map_err(Into::into)
}
