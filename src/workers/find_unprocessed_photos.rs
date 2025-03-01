use crate::common::settings::Settings;
use crate::models::images;
use crate::workers::processing_api::process_thumbnails;
use images::Entity;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tracing::info;

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
        info!("=================FindUnprocessedPhotos=======================");
        let settings = Settings::from_context(&self.ctx);

        info!(
            "📸 Starting photo processing from: {:?}",
            &settings.media_dir
        );

        let media_path = Path::new(&settings.media_dir);
        fs::create_dir_all(media_path).await?;
        // TODO make sure processed images have all thumbnails

        let existing_paths = Entity::get_relative_paths(&self.ctx.db).await?;
        let unprocessed_photos = collect_unprocessed_photos(media_path, &existing_paths).await?;

        info!("Found {} unprocessed photos", unprocessed_photos.len());
        info!("Unprocessed photos: {:?}", unprocessed_photos);

        if !unprocessed_photos.is_empty() {
            process_thumbnails(unprocessed_photos, settings).await?;
        }

        info!("✅ Successfully processed photos");
        Ok(())
    }
}

async fn collect_unprocessed_photos(
    media_path: &Path,
    existing_paths: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut unprocessed_photos = Vec::new();
    let mut entries = fs::read_dir(media_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let input_path = entry.path();
        if input_path.is_file() {
            if let Ok(relative_path) = input_path.strip_prefix(media_path) {
                let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                if !existing_paths.contains(&relative_path_str) {
                    unprocessed_photos.push(relative_path_str);
                }
            }
        }
    }
    Ok(unprocessed_photos)
}
