use crate::common;
use crate::models::images;
use images::Entity;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tracing::{info};

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
        let settings = if let Some(s) = &self.ctx.config.settings {
            common::settings::Settings::from_json(s)?
        } else {
            return Err(Error::Message("Settings not found in config".to_string()));
        };
        let media_folder = settings
            .media_folder
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Media folder not specified in settings"))
            .map_err(|e| Error::Message(e.to_string()))?;

        info!("📸 Starting photo processing from: {:?}", media_folder);

        let media_path = Path::new(media_folder);
        fs::create_dir_all(media_path).await?;

        // Get database connection
        let db = &self.ctx.db;
        // TODO make sure processed images have all thumbnails

        let existing_paths = Entity::get_relative_paths(db).await?;

        // Step 3-4: Find unprocessed photos
        let unprocessed_photos = collect_unprocessed_photos(media_path, &existing_paths).await?;

        info!("Found {} unprocessed photos", unprocessed_photos.len());
        info!("Unprocessed photos: {:?}", unprocessed_photos);
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
