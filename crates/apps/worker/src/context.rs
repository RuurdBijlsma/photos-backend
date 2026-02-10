use app_state::AppSettings;
use color_eyre::Result;
use common_services::s2s_client::S2SClient;
use media_analyzer::MediaAnalyzer;
use ml_analysis::VisualAnalyzer;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;

pub struct WorkerContext {
    pub worker_id: String,
    pub handle_analysis: bool,
    pub pool: PgPool,
    pub settings: AppSettings,
    pub media_analyzer: Arc<MediaAnalyzer>,
    pub visual_analyzer: Option<Arc<VisualAnalyzer>>,
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
        let embedder_model_id = &settings.ingest.analyzer.search.embedder_model_id.clone();
        let visual_analyzer = if handle_analysis {
            Some(Arc::new(VisualAnalyzer::new(embedder_model_id).await?))
        } else {
            None
        };
        Ok(Self {
            worker_id,
            handle_analysis,
            pool,
            settings,
            media_analyzer: Arc::new(MediaAnalyzer::builder().build().await?),
            visual_analyzer,
            s2s_client: S2SClient::new(Client::new()),
        })
    }
}
