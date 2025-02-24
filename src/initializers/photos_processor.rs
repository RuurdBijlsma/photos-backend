// src/initializers/photos_processor.rs
use crate::workers::find_unprocessed_photos::{FindUnprocessedPhotosWorker, WorkerArgs};
use loco_rs::prelude::*;

pub struct PhotosProcessorInitializer;

#[async_trait]
impl Initializer for PhotosProcessorInitializer {
    fn name(&self) -> String {
        "photos-processor".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        FindUnprocessedPhotosWorker::perform_later(ctx, WorkerArgs {}).await?;

        Ok(())
    }
}
