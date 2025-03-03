use crate::api::thumbnail_api;
use crate::common::settings::Settings;
use crate::workers::analyze_images;
use crate::workers::analyze_images::AnalyzeImagesWorker;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

pub struct GenerateThumbnailsWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {
    pub images: Vec<String>,
}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for GenerateThumbnailsWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: WorkerArgs) -> Result<()> {
        info!("======================= GenerateThumbnails =======================");

        let settings = Settings::from_context(&self.ctx);
        thumbnail_api::generate_thumbnails(args.images.clone(), &settings.processing_api_url)
            .await
            .map_err(|e| Error::Message(e.to_string()))?;

        info!("✅ Successfully Generated Thumbnails");

        AnalyzeImagesWorker::perform_later(
            &self.ctx,
            analyze_images::WorkerArgs {
                image: args.images[0].to_string(),
            },
        )
        .await?;

        Ok(())
    }
}
