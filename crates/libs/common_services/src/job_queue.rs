use crate::database::jobs::JobType;
use app_state::AppSettings;
use bon::builder;
use color_eyre::eyre::Result;
use serde::Serialize;
use serde_json::to_value;
use sqlx::{PgPool, PgTransaction};
use tracing::{info, warn};

/// Enqueues a job for a specific file, such as ingestion or removal.
///
/// # Errors
///
/// Returns an error if the database transaction fails.
#[builder]
pub async fn enqueue_job<T: Serialize + Send + Sync>(
    #[builder(start_fn)] pool: &PgPool,
    #[builder(start_fn)] settings: &AppSettings,
    #[builder(start_fn)] job_type: JobType,
    #[builder(into)] relative_path: Option<String>,
    user_id: Option<i32>,
    payload: Option<&T>,
) -> Result<bool> {
    let json_payload = payload.and_then(|p| to_value(p).ok());

    let mut tx = pool.begin().await?;

    if let Some(rel_path) = &relative_path {
        per_job_logic(&mut tx, job_type, rel_path).await?;
    }

    let is_video = relative_path.as_ref().is_some_and(|p| {
        settings
            .ingestion
            .is_video_file(&settings.ingestion.media_folder.join(p))
    });
    let priority = job_type.get_priority(is_video);

    let result = sqlx::query!(
        r#"
        INSERT INTO jobs (relative_path, job_type, priority, user_id, payload)
        VALUES ($1, $2, $3, $4, $5)
        -- THIS PART MUST MATCH THE INDEX DEFINITION EXACTLY
        ON CONFLICT (job_type, coalesce(user_id, -1), coalesce(md5(payload::text), ''), coalesce(relative_path, ''))
        WHERE (status IN ('queued', 'running'))
        DO NOTHING
        "#,
        relative_path.as_deref(),
        job_type as JobType,
        priority,
        user_id,
        json_payload,
    )
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if result.rows_affected() == 0 {
        warn!(
            "Not enqueueing {:?} job {:?}, an active one already exists.",
            job_type, relative_path
        );
        return Ok(false);
    }

    info!(
        "Enqueued {:?} job {:?}, user_id: {:?}, payload: {:?}",
        job_type, relative_path, user_id, json_payload
    );

    Ok(true)
}

/// Enqueues a full ingest and analysis job for a given file.
///
/// # Errors
///
/// Returns an error if any of the database operations fail.
pub async fn enqueue_full_ingest(
    pool: &PgPool,
    settings: &AppSettings,
    relative_path: &str,
    user_id: i32,
) -> Result<()> {
    enqueue_job::<()>(pool, settings, JobType::Ingest)
        .relative_path(relative_path)
        .user_id(user_id)
        .call()
        .await?;
    enqueue_job::<()>(pool, settings, JobType::Analysis)
        .relative_path(relative_path)
        .user_id(user_id)
        .call()
        .await?;

    Ok(())
}

/// Applies some job logic specific to each job type.
///
/// * Enqueueing a remove job means existing ingest/analysis jobs for that file are be cancelled.
/// * Enqueueing an ingest/analysis job means existing remove jobs for that file are cancelled.
pub async fn per_job_logic(
    tx: &mut PgTransaction<'_>,
    job_type: JobType,
    relative_path: &str,
) -> Result<()> {
    match job_type {
        JobType::Remove => cancel_ingest_analysis_jobs(tx, relative_path).await?,
        JobType::Ingest | JobType::Analysis => cancel_remove_jobs(tx, relative_path).await?,
        _ => (),
    }

    Ok(())
}

/// Cancel remove jobs for given `relative_path`.
async fn cancel_remove_jobs(tx: &mut PgTransaction<'_>, relative_path: &str) -> Result<()> {
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'cancelled'
        WHERE
            relative_path = $1
            AND status IN ('queued', 'running')
            AND job_type IN ('remove')
        "#,
        relative_path
    )
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            "Cancelled {} queued/running remove job(s) for file: {}",
            result.rows_affected(),
            relative_path
        );
    }

    Ok(())
}

/// Cancels any queued ingest or analysis jobs.
///
/// # Errors
///
/// Returns an error if any of the database queries or the transaction commit fails.
async fn cancel_ingest_analysis_jobs(
    tx: &mut PgTransaction<'_>,
    relative_path: &str,
) -> Result<()> {
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'cancelled'
        WHERE
            relative_path = $1
            AND status IN ('queued', 'running')
            AND job_type IN ('ingest', 'analysis')
        "#,
        relative_path
    )
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() > 0 {
        info!(
            "Cancelled {} queued/running ingest/analysis job(s) for file: {}",
            result.rows_affected(),
            relative_path
        );
    }

    Ok(())
}
