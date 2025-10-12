use std::time::Duration;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use tracing::info;

pub async fn clean_db(pool: &PgPool) -> color_eyre::Result<()> {
    let result = sqlx::query!("DELETE FROM refresh_token WHERE expires_at < $1", Utc::now() - Duration::from_secs(1 * 60 * 60))
        .execute(pool)
        .await?;

    info!("Cleaned up refresh token table, {} rows affected", result.rows_affected());

    Ok(())
}
