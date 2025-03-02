use crate::api::analyze_api;
use crate::common::settings::Settings;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

pub struct AnalyzeImagesWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {
    pub image: String,
}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for AnalyzeImagesWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: WorkerArgs) -> Result<()> {
        info!("======================= ProcessImages =======================");

        let settings = Settings::from_context(&self.ctx);
        let result = analyze_api::process_media(args.image.clone(), &settings)
            .await
            .map_err(|e| Error::Message(e.to_string()))?;

        info!("✅ Successfully Analyzed Image {}", args.image);
        Ok(())
    }
}
