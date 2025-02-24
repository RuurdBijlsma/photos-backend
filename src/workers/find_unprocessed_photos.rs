use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
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
        let input_path = "input_images";
        println!("📸 Starting photo processing from: {}", input_path);

        // Create output directory if it doesn't exist
        fs::create_dir_all(&input_path).await?;

        let mut entries = fs::read_dir(&input_path).await?;

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
