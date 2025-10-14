use sqlx::PgPool;
use tokio::task::JoinHandle;

pub fn start_heartbeat_loop(pool: &PgPool, job_id: i64) -> JoinHandle<()> {
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));
        loop {
            interval.tick().await;
            let result = sqlx::query!(
                "UPDATE jobs SET last_heartbeat = now() WHERE id = $1 AND status = 'running'",
                job_id
            )
            .execute(&pool_clone)
            .await;

            if result.is_err() || result.unwrap().rows_affected() == 0 {
                break;
            }
        }
    })
}
