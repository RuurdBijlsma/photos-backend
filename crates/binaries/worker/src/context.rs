use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use ml_analysis::VisualAnalyzer;
use sqlx::PgPool;
use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Mutex;
use common_services::s2s_client::S2SClient;

pub struct WorkerContext {
    pub pool: PgPool,
    pub worker_id: String,
    pub media_analyzer: Arc<Mutex<MediaAnalyzer>>,
    pub visual_analyzer: VisualAnalyzer,
    pub handle_analysis: bool,
    pub s2s_client: S2SClient,
}

impl WorkerContext {
    /// Creates a new instance of `WorkerContext`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the creation of `MediaAnalyzer` or `VisualAnalyzer` fails.
    pub async fn new(pool: PgPool, worker_id: String, handle_analysis: bool) -> Result<Self> {
        Ok(Self {
            pool,
            worker_id,
            media_analyzer: Arc::new(Mutex::new(MediaAnalyzer::builder().build().await?)),
            visual_analyzer: VisualAnalyzer::new()?,
            s2s_client: S2SClient::new(Client::new()),
            handle_analysis,
        })
    }
}
