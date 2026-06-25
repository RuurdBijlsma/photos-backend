// File: crates/libs/common_services/src/api/jobs/service.rs

use crate::api::app_error::AppError;
use crate::api::jobs::interfaces::{JobInfo, JobsResponse};
use crate::database::jobs::{JobStatus, JobType};
use sqlx::PgPool;

pub async fn get_job_overview(pool: &PgPool) -> Result<JobsResponse, AppError> {
    let (running, queued, failed, cancelled, recently_done) = tokio::try_join!(
        // 1. Fetch running jobs (limit 50, sorted by started_at DESC)
        sqlx::query_as!(
            JobInfo,
            r#"
            SELECT
                id,
                relative_path,
                user_id,
                job_type AS "job_type: JobType",
                payload AS "payload?: serde_json::Value",
                priority,
                status AS "status: JobStatus",
                attempts,
                dependency_attempts,
                max_attempts,
                owner,
                started_at,
                finished_at,
                created_at,
                scheduled_at,
                last_heartbeat,
                last_error
            FROM jobs
            WHERE status = 'running'
            ORDER BY started_at DESC NULLS LAST, created_at DESC
            LIMIT 50
            "#
        )
        .fetch_all(pool),

        // 2. Fetch queued jobs (limit 50, sorted by claim priority)
        sqlx::query_as!(
            JobInfo,
            r#"
            SELECT
                id,
                relative_path,
                user_id,
                job_type AS "job_type: JobType",
                payload AS "payload?: serde_json::Value",
                priority,
                status AS "status: JobStatus",
                attempts,
                dependency_attempts,
                max_attempts,
                owner,
                started_at,
                finished_at,
                created_at,
                scheduled_at,
                last_heartbeat,
                last_error
            FROM jobs
            WHERE status = 'queued'
            ORDER BY priority ASC, scheduled_at ASC, created_at ASC
            LIMIT 50
            "#
        )
        .fetch_all(pool),

        // 3. Fetch failed jobs (limit 50, sorted by finished_at DESC)
        sqlx::query_as!(
            JobInfo,
            r#"
            SELECT
                id,
                relative_path,
                user_id,
                job_type AS "job_type: JobType",
                payload AS "payload?: serde_json::Value",
                priority,
                status AS "status: JobStatus",
                attempts,
                dependency_attempts,
                max_attempts,
                owner,
                started_at,
                finished_at,
                created_at,
                scheduled_at,
                last_heartbeat,
                last_error
            FROM jobs
            WHERE status = 'failed'
            ORDER BY finished_at DESC NULLS LAST, created_at DESC
            LIMIT 50
            "#
        )
        .fetch_all(pool),

        // 4. Fetch cancelled jobs (limit 50, sorted by finished_at DESC)
        sqlx::query_as!(
            JobInfo,
            r#"
            SELECT
                id,
                relative_path,
                user_id,
                job_type AS "job_type: JobType",
                payload AS "payload?: serde_json::Value",
                priority,
                status AS "status: JobStatus",
                attempts,
                dependency_attempts,
                max_attempts,
                owner,
                started_at,
                finished_at,
                created_at,
                scheduled_at,
                last_heartbeat,
                last_error
            FROM jobs
            WHERE status = 'cancelled'
            ORDER BY finished_at DESC NULLS LAST, created_at DESC
            LIMIT 50
            "#
        )
        .fetch_all(pool),

        // 5. Fetch 30 most recent completed done jobs (sorted by finished_at DESC)
        sqlx::query_as!(
            JobInfo,
            r#"
            SELECT
                id,
                relative_path,
                user_id,
                job_type AS "job_type: JobType",
                payload AS "payload?: serde_json::Value",
                priority,
                status AS "status: JobStatus",
                attempts,
                dependency_attempts,
                max_attempts,
                owner,
                started_at,
                finished_at,
                created_at,
                scheduled_at,
                last_heartbeat,
                last_error
            FROM jobs
            WHERE status = 'done'
            ORDER BY finished_at DESC NULLS LAST, created_at DESC
            LIMIT 30
            "#
        )
        .fetch_all(pool)
    )?;

    Ok(JobsResponse {
        running,
        queued,
        failed,
        cancelled,
        recently_done,
    })
}