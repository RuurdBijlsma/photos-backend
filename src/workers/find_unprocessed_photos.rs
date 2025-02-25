use crate::common;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
pub struct FindUnprocessedPhotosWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for FindUnprocessedPhotosWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    async fn perform(&self, _args: WorkerArgs) -> Result<()> {
        println!("=================FindUnprocessedPhotos=======================");
        let settings = if let Some(settings) = &self.ctx.config.settings {
            common::settings::Settings::from_json(settings)?
        } else {
            return Err(anyhow::anyhow!("Settings not found in config"))
                .map_err(|e| loco_rs::Error::Message(e.to_string()));
        };
        let media_folder = settings.media_folder.as_deref().unwrap();
        println!("📸 Starting photo processing from: {:?}", media_folder);

        let media_path = Path::new(media_folder);
        fs::create_dir_all(media_path).await?;

        let mut entries = fs::read_dir(&media_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let input_path = entry.path();
            if input_path.is_file() {
                println!("{input_path:?}");
            }
        }
        println!("✅ Successfully processed photos");
        Ok(())
    }
}
