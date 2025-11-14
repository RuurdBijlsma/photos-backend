use std::time::Duration;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use crate::get_settings::settings;

/// Run migrations and get a database connection pool.
/// # Errors
///
/// * `env::var` can return an error if `DATABASE_URL` is not found in the environment.
/// * `PgPool::connect` can return an error if the database connection fails.
/// * `sqlx::migrate` can return an error if migrations fail.
pub async fn get_db_pool() -> color_eyre::Result<Pool<Postgres>> {
    let db_settings = &settings().database;
    let database_url = &db_settings.url;
    info!("Connecting to database.");
    let pool = PgPoolOptions::new()
        .max_connections(db_settings.max_connections)
        .min_connections(db_settings.min_connection)
        .max_lifetime(Duration::from_secs(db_settings.max_lifetime))
        .idle_timeout(Duration::from_secs(db_settings.idle_timeout))
        .acquire_timeout(Duration::from_secs(db_settings.acquire_timeout))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;
    Ok(pool)
}