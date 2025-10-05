use sqlx::Pool;
use color_eyre::eyre::Result;
use sqlx::{PgPool, Postgres};
use std::env;

pub async fn get_db_pool() -> Result<Pool<Postgres>> {
    dotenv::from_path(".env").ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    return Ok(pool);
}







