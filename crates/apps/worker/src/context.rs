use app_state::AppSettings;
use color_eyre::Result;
use common_services::s2s_client::S2SClient;
use media_analyzer::MediaAnalyzer;
use ml_analysis::VisualAnalyzer;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct WorkerContext {
    pub worker_id: String,
    pub handle_analysis: bool,
    pub pool: PgPool,
    pub settings: AppSettings,
    pub media_analyzer: Arc<Mutex<MediaAnalyzer>>,
    pub visual_analyzer: Arc<VisualAnalyzer>,
    pub s2s_client: S2SClient,
}

impl WorkerContext {
    /// Creates a new instance of `WorkerContext`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the creation of `MediaAnalyzer` or `VisualAnalyzer` fails.
    pub async fn new(
        pool: PgPool,
        settings: AppSettings,
        worker_id: String,
        handle_analysis: bool,
    ) -> Result<Self> {
        Ok(Self {
            worker_id,
            handle_analysis,
            pool,
            settings,
            media_analyzer: Arc::new(Mutex::new(MediaAnalyzer::builder().build().await?)),
            visual_analyzer: Arc::new(VisualAnalyzer::new()?),
            s2s_client: S2SClient::new(Client::new()),
        })
    }
}
