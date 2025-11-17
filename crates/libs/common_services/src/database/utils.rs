use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use tracing::info;
use app_state::constants;

/// Run migrations and get a database connection pool.
/// # Errors
///
/// * `env::var` can return an error if `DATABASE_URL` is not found in the environment.
/// * `PgPool::connect` can return an error if the database connection fails.
/// * `sqlx::migrate` can return an error if migrations fail.
pub async fn get_db_pool(database_url: &str) -> color_eyre::Result<Pool<Postgres>> {
    info!("Connecting to database.");
    let db_config = &constants().database;
    let pool = PgPoolOptions::new()
        .max_connections(db_config.max_connections)
        .min_connections(db_config.min_connection)
        .max_lifetime(Duration::from_secs(db_config.max_lifetime))
        .idle_timeout(Duration::from_secs(db_config.idle_timeout))
        .acquire_timeout(Duration::from_secs(db_config.acquire_timeout))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;
    Ok(pool)
}
