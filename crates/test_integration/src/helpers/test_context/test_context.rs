use crate::helpers::test_context::context_utils::{
    create_test_database, create_test_settings, force_drop_db,
};
use app_state::{
    load_constants_from_path, load_settings_from_path, AppConstants, AppSettings, CONSTANTS,
};
use color_eyre::eyre::{eyre, Result};
use reqwest::Client;
use sqlx::PgPool;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

pub fn init_test_constants(constants: AppConstants) {
    if CONSTANTS.set(constants).is_err() {
        info!("AppConstants were already initialized by another test.");
    }
}

/// The main context for our integration tests.
#[allow(dead_code)]
pub struct TestContext {
    pub pool: PgPool,
    pub settings: AppSettings,
    pub http_client: Client,
    // Private fields for robust cleanup on Drop
    db_name: String,
    management_pool: PgPool,
    media_dir: TempDir,
    thumbnail_dir: TempDir,
    api_handle: JoinHandle<()>,
    worker_handle: JoinHandle<()>,
    watcher_handle: JoinHandle<()>,
}

impl TestContext {
    /// Sets up the entire test environment, including a dedicated database and background services.
    pub async fn new() -> Result<Self> {
        info!("Setting up test environment...");

        // Load base settings to get initial database connection info
        let settings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/settings.yaml")
            .canonicalize()?;
        let base_settings = load_settings_from_path(&settings_path, false)?;
        let test_constants = load_constants_from_path(&settings_path)?;
        init_test_constants(test_constants);

        // 1. Set up the dedicated test database
        let db_name = "test_db".to_owned();
        let (main_pool, management_pool) =
            create_test_database(&base_settings.secrets.database_url, &db_name).await?;

        // 2. Generate the final settings for this test run
        let (settings, media_dir, thumbnail_dir) = create_test_settings(&db_name, &base_settings)?;

        // 3. Spawn application components as background tasks
        let (api_handle, worker_handle, watcher_handle) =
            Self::spawn_services(&main_pool, &settings);

        // 4. Wait for the API to be ready to accept traffic
        let http_client = Client::new();
        Self::wait_for_healthy_api(&settings, &http_client).await?;

        info!("Test environment is ready.");
        Ok(Self {
            pool: main_pool,
            settings,
            http_client,
            db_name,
            management_pool,
            media_dir,
            thumbnail_dir,
            api_handle,
            worker_handle,
            watcher_handle,
        })
    }

    /// Spawns the API, worker, and watcher services as background tokio tasks.
    fn spawn_services(
        pool: &PgPool,
        settings: &AppSettings,
    ) -> (JoinHandle<()>, JoinHandle<()>, JoinHandle<()>) {
        // Spawn API server
        let api_pool = pool.clone();
        let api_settings = settings.clone();
        let api_handle = tokio::spawn(async move {
            if let Err(e) = api::serve(api_pool, api_settings).await {
                error!("API server failed: {}", e);
            }
        });

        // Spawn Worker
        let worker_pool = pool.clone();
        let worker_setting = settings.clone();
        let worker_handle = tokio::spawn(async move {
            if let Err(e) = worker::worker::create_worker(worker_pool, worker_setting, true).await {
                error!("Worker failed: {}", e);
            }
        });

        // Spawn Watcher
        let watcher_pool = pool.clone();
        let watcher_setting = settings.clone();
        let watcher_handle = tokio::spawn(async move {
            if let Err(e) = watcher::watcher::start_watching(watcher_pool, watcher_setting).await {
                error!("Watcher failed: {}", e);
            }
        });

        (api_handle, worker_handle, watcher_handle)
    }

    /// Polls the `/health` endpoint until it receives a successful response or times out.
    async fn wait_for_healthy_api(settings: &AppSettings, http_client: &Client) -> Result<()> {
        for attempt in 1..=20 {
            info!("Health check attempt {}...", attempt);
            let health_url = format!("{}/health", &settings.api.public_url);
            match http_client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    info!("API is healthy!");
                    return Ok(());
                }
                Ok(response) => {
                    warn!(
                        "API health check returned non-success status: {}",
                        response.status()
                    );
                }
                Err(e) => {
                    warn!("API health check failed: {:?}. Retrying...", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(eyre!(
            "API did not become healthy within the timeout period."
        ))
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Abort background tasks
        self.api_handle.abort();
        self.worker_handle.abort();
        self.watcher_handle.abort();

        // Asynchronously drop the test database
        let db_name = self.db_name.clone();
        let pool = self.management_pool.clone();
        tokio::spawn(async move {
            info!("Dropping test database: {}", db_name);
            force_drop_db(&pool, &db_name)
                .await
                .expect("Failed to clean up DB.");
        });

        info!("Teardown complete.");
    }
}
