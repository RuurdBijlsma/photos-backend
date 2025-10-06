use crate::get_media_dir;
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::path::Path;
use std::path::absolute;

pub fn get_relative_path_str(file: impl AsRef<Path>) -> color_eyre::Result<String> {
    let file_abs = absolute(file)?;
    let relative_path = file_abs.strip_prefix(get_media_dir())?;
    let relative_path_str = relative_path.to_string_lossy().to_string();
    Ok(relative_path_str)
}

pub fn nice_id(length: usize) -> String {
    const URL_SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    (0..length)
        .map(|_| {
            let idx = rand::random_range(0..URL_SAFE.len());
            URL_SAFE[idx] as char
        })
        .collect()
}

pub async fn get_db_pool() -> color_eyre::Result<Pool<Postgres>> {
    dotenv::from_path(".env").ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    Ok(pool)
}
