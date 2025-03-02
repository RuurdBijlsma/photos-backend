use crate::common::settings::Settings;
use crate::models::images;
use crate::workers::generate_thumbnails;
use crate::workers::generate_thumbnails::GenerateThumbnailsWorker;
use images::Entity;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tracing::{error, info};
use walkdir::WalkDir;
use crate::common::image_utils::{is_image_file, is_video_file, normalize_path};

pub struct FindUnprocessedImagesWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for FindUnprocessedImagesWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, _args: WorkerArgs) -> Result<()> {
        info!("=================FindUnprocessedWorker=======================");
        let settings = Settings::from_context(&self.ctx);

        info!(
            "📸 Starting image processing from: {:?}",
            &settings.media_dir
        );

        let media_path = Path::new(&settings.media_dir);
        fs::create_dir_all(media_path).await?;
        // TODO make sure processed images have all thumbnails

        let existing_paths = Entity::get_relative_paths(&self.ctx.db).await?;
        let unprocessed_images = collect_unprocessed_images(media_path, &existing_paths).await?;

        info!("Found {} unprocessed images", unprocessed_images.len());
        info!("Unprocessed images: {:?}", unprocessed_images);

        if !unprocessed_images.is_empty() {
            GenerateThumbnailsWorker::perform_later(
                &self.ctx,
                generate_thumbnails::WorkerArgs {
                    images: unprocessed_images,
                },
            )
            .await?;
        }

        Ok(())
    }
}

async fn collect_unprocessed_images(
    media_path: &Path,
    existing_paths: &HashSet<String>,
) -> Result<Vec<String>, loco_rs::Error> {
    // Clone necessary data to move into the blocking closure.
    let media_path = media_path.to_path_buf();
    let existing_paths = existing_paths.clone();

    let result = tokio::task::spawn_blocking(move || {
        // Iterate through the directory entries.
        return WalkDir::new(&media_path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                if is_image_file(path) || is_video_file(path) {
                    path.strip_prefix(&media_path).ok().map(normalize_path)
                } else {
                    None
                }
            })
            .filter(|normalized| !existing_paths.contains(normalized))
            .collect::<Vec<String>>();
    })
    .await
    .map_err(|e| loco_rs::Error::Message(e.to_string()))
    .map_err(|e| loco_rs::Error::Message(e.to_string()))?;

    Ok(result)
}
