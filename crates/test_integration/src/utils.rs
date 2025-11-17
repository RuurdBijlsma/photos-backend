use std::net::TcpListener;
use app_state::{load_settings_from_path, AppSettings};
use color_eyre::eyre::Result;
use common_services::database::get_db_pool;
use sqlx::{Executor, PgPool};
use std::path::Path;
use tempfile::TempDir;
use tracing::info;
use url::Url;

pub fn create_test_settings(
    database_name: &str,
    base_settings: &AppSettings,
) -> Result<(AppSettings, TempDir, TempDir)> {
    // 1. Load base settings from the test configuration file.
    let mut settings = base_settings.clone();

    // 2. Create temporary directories for media and thumbnails.
    let media_dir = TempDir::new()?;
    let thumbnail_dir = TempDir::new()?;
    let port = get_free_port();
    settings.api.port = port as u32;
    settings.api.public_url = format!("http://localhost:{}", port);
    settings.ingest.media_root = media_dir.path().to_path_buf();
    settings.ingest.media_root_canon = media_dir.path().to_path_buf(); // Also set the canonical path
    settings.ingest.thumbnail_root = thumbnail_dir.path().to_path_buf();

    // 3. Update the database URL to point to our unique test database.
    let mut db_url = Url::parse(&settings.secrets.database_url)?;
    db_url.set_path(&format!("/{database_name}"));
    settings.secrets.database_url = db_url.to_string();

    println!("DB URL: {}", settings.secrets.database_url);

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
    force_drop_db(&management_pool, &database_name)
        .await
        .expect("Failed to clean up DB.");

    // 2. Create the new test database.
    management_pool
        .execute(format!("CREATE DATABASE \"{database_name}\"").as_str())
        .await?;

    // 3. Connect to the newly created test database.
    let mut test_db_url = Url::parse(base_database_url)?;
    test_db_url.set_path(&format!("/{database_name}"));
    let main_pool = get_db_pool(test_db_url.as_str()).await?;

    // 4. Run migrations on the test database.
    sqlx::migrate!("../../migrations").run(&main_pool).await?;
    info!("Finished database migrations for {}", database_name);

    Ok((main_pool, management_pool))
}

pub async fn force_drop_db(management_pool: &PgPool, db_name: &str) -> Result<()> {
    let _ = management_pool
        .execute(format!("DROP DATABASE \"{db_name}\" WITH (FORCE)").as_str())
        .await;
    Ok(())
}

pub fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}