use crate::{ApiSettings, IngestSettings, LoggingSettings, RawSettings, SecretSettings, MakeRelativePath};
use color_eyre::Result;
use serde::Deserialize;
use sqlx::{Executor, Postgres};
use std::path::{Path, absolute};

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub ingest: IngestSettings,
    pub logging: LoggingSettings,
    pub api: ApiSettings,
    pub secrets: SecretSettings,
}

impl From<RawSettings> for AppSettings {
    fn from(raw: RawSettings) -> Self {
        let thumbnails_root =
            absolute(&raw.ingest.thumbnail_folder).expect("Invalid thumbnail_folder");
        let media_root = absolute(&raw.ingest.media_folder).expect("Invalid media_folder");
        let ingest = IngestSettings {
            media_folder: media_root,
            thumbnail_folder: thumbnails_root,
            analyzer: raw.ingest.analyzer,
            file_detection: raw.ingest.file_detection,
            thumbnails: raw.ingest.thumbnails,
        };

        Self {
            ingest,
            logging: raw.logging,
            api: raw.api,
            secrets: raw.secrets,
        }
    }
}

impl IngestSettings {
    // stuff that needs multiple settings (otherwise just make it a standalone function).

    #[must_use]
    pub fn is_media_file(&self, file: &Path) -> bool {
        let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
            return false;
        };
        let is_photo = self.file_detection.photo_extensions.contains(&extension);
        let is_video = self.file_detection.video_extensions.contains(&extension);
        is_photo || is_video
    }

    #[must_use]
    pub fn is_photo_file(&self, file: &Path) -> bool {
        let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
            return false;
        };
        self.file_detection.photo_extensions.contains(&extension)
    }

    #[must_use]
    pub fn is_video_file(&self, file: &Path) -> bool {
        let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_lowercase()) else {
            return false;
        };
        self.file_detection.video_extensions.contains(&extension)
    }

    /// Verifies if all expected thumbnails for a given media file exist on disk.
    /// # Errors
    ///
    /// This function's signature returns a Result, but the current implementation does not produce any errors.
    pub fn thumbs_exist(&self, file: &Path, thumb_sub_folder: &str) -> Result<bool> {
        let is_photo = self.is_photo_file(file);
        let is_video = self.is_video_file(file);
        let photo_thumb_ext = &self.thumbnails.thumbnail_extension;
        let video_thumb_ext = &self.thumbnails.video_options.extension;
        let mut should_exist: Vec<String> = vec![];

        if is_photo || is_video {
            // Both photo and video should have a thumbnail for each entry in .heights.
            for h in &self.thumbnails.heights {
                should_exist.push(format!("{h}p.{photo_thumb_ext}"));
            }
        }
        if is_video {
            for p in &self.thumbnails.video_options.percentages {
                should_exist.push(format!("{p}_percent.{photo_thumb_ext}"));
            }
            for x in &self.thumbnails.video_options.transcode_outputs {
                let height = x.height;
                should_exist.push(format!("{height}p.{video_thumb_ext}"));
            }
        }

        let thumb_dir = self.thumbnail_folder.join(thumb_sub_folder);
        for thumb_filename in should_exist {
            let thumb_file_path = thumb_dir.join(thumb_filename.clone());
            if !thumb_file_path.exists() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Checks if a file has already been ingested by verifying its database record and thumbnail existence.
    /// # Errors
    ///
    /// * Can return an error from `thumbs_exist` if checking for thumbnails fails.
    pub async fn file_is_ingested(
        &self,
        executor: impl Executor<'_, Database = Postgres>,
        file: &Path,
    ) -> Result<bool> {
        // Media item existence check:
        let Ok(relative_path_str) = file.make_relative(&self.media_folder) else {
            return Ok(false);
        };
        let Ok(media_item_id) = sqlx::query_scalar!(
            "SELECT id FROM media_item WHERE relative_path = $1",
            relative_path_str
        )
        .fetch_optional(executor)
        .await
        else {
            return Ok(false);
        };
        let Some(media_item_id) = media_item_id else {
            return Ok(false);
        };
        // media item exists, check thumbnails existence
        let exist = self.thumbs_exist(file, &media_item_id)?;
        Ok(exist)
    }
}
