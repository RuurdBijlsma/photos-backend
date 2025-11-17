use crate::utils::{create_test_database, create_test_settings};
use app_state::{load_settings_from_path, AppSettings};
use color_eyre::Result;
use common_services::utils::nice_id;
use sqlx::{Executor, PgPool};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// The main context for our integration tests.
pub struct TestContext {
    pub pool: PgPool,
    pub settings: AppSettings,
    pub http_client: reqwest::Client,
    // Private fields for robust cleanup on Drop
    _db_name: String,
    _management_pool: PgPool,
    _media_dir: TempDir,
    _thumbnail_dir: TempDir,
    _api_handle: JoinHandle<()>,
    _worker_handle: JoinHandle<()>,
    _watcher_handle: JoinHandle<()>,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        info!("Setting up test environment...");

        // Use a temporary base settings object to get the base DB URL
        let settings_path = Path::new("crates/test_integration/assets/settings.yaml");
        let base_settings = load_settings_from_path(settings_path, true)?;
        let database_name = format!("test_{}", nice_id(8));

        // Set up the dedicated test database
        let (main_pool, management_pool) =
            create_test_database(&base_settings.secrets.database_url, &database_name).await?;

        // Generate the final settings for this test run
        let (settings, media_dir, thumbnail_dir) = create_test_settings(&database_name, settings_path)?;

        // Spawn application components as background tasks
        let api_pool = main_pool.clone();
        let api_settings = settings.clone();
        let api_handle = tokio::spawn(async move {
            if let Err(e) = api::serve(api_pool, api_settings).await {
                error!("API server failed: {}", e);
            }
        });

        let worker_pool = main_pool.clone();
        let worker_setting = settings.clone();
        let worker_handle = tokio::spawn(async move {
            if let Err(e) = worker::worker::create_worker(worker_pool, worker_setting, true).await
            {
                error!("Worker failed: {}", e);
            }
        });

        let watcher_pool = main_pool.clone();
        let watcher_setting = settings.clone();
        let watcher_handle = tokio::spawn(async move {
            if let Err(e) = watcher::watcher::start_watching(watcher_pool, watcher_setting).await {
                error!("Watcher failed: {}", e);
            }
        });

        info!("Waiting for components to start...");
        tokio::time::sleep(Duration::from_secs(2)).await;
        info!("Test environment is ready.");

        Ok(Self {
            pool: main_pool,
            settings,
            http_client: reqwest::Client::new(),
            _db_name: database_name,
            _management_pool: management_pool,
            _media_dir: media_dir,
            _thumbnail_dir: thumbnail_dir,
            _api_handle: api_handle,
            _worker_handle: worker_handle,
            _watcher_handle: watcher_handle,
        })
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        info!("Tearing down test environment...");

        self._api_handle.abort();
        self._worker_handle.abort();
        self._watcher_handle.abort();

        let db_name = self._db_name.clone();
        let pool = self._management_pool.clone();
        tokio::spawn(async move {
            info!("Dropping test database: {}", db_name);
            // CRITICAL FIX: Use an unprepared query for DROP DATABASE.
            let query = format!("DROP DATABASE \"{}\" WITH (FORCE)", db_name);
            pool.execute(query.as_str()).await.unwrap_or_else(|e| {
                panic!("Failed to drop test database {}: {}", db_name, e);
            });
        });

        info!("Teardown complete.");
    }
}
