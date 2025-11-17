use app_state::{load_settings_from_path, AppSettings};
use color_eyre::eyre::Result;
use common_services::database::get_db_pool;
use sqlx::{Executor, PgPool};
use std::path::Path;
use tempfile::TempDir;
use tracing::info;
use url::Url;

pub fn create_test_settings(database_name: &str, settings_path: &Path) -> Result<(AppSettings, TempDir, TempDir)> {
    // 1. Load base settings from the test configuration file.
    let mut settings = load_settings_from_path(settings_path, false)?;

    // 2. Create temporary directories for media and thumbnails.
    let media_dir = TempDir::new()?;
    let thumbnail_dir = TempDir::new()?;
    settings.ingest.media_root = media_dir.path().to_path_buf();
    settings.ingest.media_root_canon = media_dir.path().to_path_buf(); // Also set the canonical path
    settings.ingest.thumbnail_root = thumbnail_dir.path().to_path_buf();

    // 3. Update the database URL to point to our unique test database.
    let mut db_url = Url::parse(&settings.secrets.database_url)?;
    db_url.set_path(&format!("/{database_name}"));
    settings.secrets.database_url = db_url.to_string();

    Ok((settings, media_dir, thumbnail_dir))
}

pub async fn create_test_database(
    base_database_url: &str,
    database_name: &str,
) -> Result<(PgPool, PgPool)> {
    // 1. Connect to the default 'postgres' database to manage other databases.
    let mut management_db_url = Url::parse(base_database_url)?;
    management_db_url.set_path("/postgres");
    let management_pool = get_db_pool(management_db_url.as_str()).await?;

    // 2. Create the new test database.
    management_pool
        .execute(format!("CREATE DATABASE \"{database_name}\"").as_str())
        .await?;
    info!("Created test database: {}", database_name);

    // 3. Connect to the newly created test database.
    let mut test_db_url = Url::parse(base_database_url)?;
    test_db_url.set_path(&format!("/{database_name}"));
    let main_pool = get_db_pool(test_db_url.as_str()).await?;

    // 4. Run migrations on the test database.
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&main_pool)
        .await?;
    sqlx::migrate!("../../migrations").run(&main_pool).await?;
    info!("Finished database migrations for {}", database_name);

    Ok((main_pool, management_pool))
}