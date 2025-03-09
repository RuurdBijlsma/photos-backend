pub use super::_entities::images::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::MediaAnalyzerOutput;
use crate::common::image_utils::parse_iso_datetime;
use crate::models::_entities::images;
use crate::models::{gps, metadata, tags, visual_features, weather};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::SelectColumns;
use std::collections::HashSet;
use std::path::Path;

pub type Images = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
    /// Create Image model based on `MediaAnalyzerOutput`, and store it in db.
    ///
    /// # Panics
    /// if filename can't be extracted from `media_path`.
    ///
    /// # Errors
    /// * If datetime string can't be parsed.
    /// * If an INSERT fails.
    /// * If querying an existing location fails.
    pub async fn create_from_analysis<C>(
        db: &C,
        user_id: i32,
        image_path: &str,
        result: MediaAnalyzerOutput,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let filename = Path::new(image_path).file_name().unwrap().to_str().unwrap();

        // Datetime parsing
        let datetime_local = parse_iso_datetime(&result.image_data.time.datetime_local)
            .map_err(|e| DbErr::Custom(e.to_string()))?;
        let datetime_utc = result
            .image_data
            .time
            .datetime_utc
            .as_ref()
            .and_then(|s| parse_iso_datetime(s).ok());

        // Create main image record
        let exif = result.image_data.exif.clone();
        let image = Self {
            filename: Set(filename.to_string()),
            relative_path: Set(image_path.to_string()),
            width: Set(exif.width),
            height: Set(exif.height),
            duration: Set(exif.duration),
            format: Set(exif.format),
            size_bytes: Set(exif.size_bytes),
            datetime_local: Set(datetime_local),
            datetime_utc: Set(datetime_utc),
            datetime_source: Set(result.image_data.time.datetime_source),
            timezone_name: Set(result.image_data.time.timezone_name),
            timezone_offset: Set(result.image_data.time.timezone_offset),
            user_id: Set(user_id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // GPS data insertion
        if let Some(gps_data) = result.image_data.gps {
            gps::ActiveModel::create_from_analysis(db, gps_data, image.id.clone()).await?;
        }

        // Metadata insertion
        metadata::ActiveModel::create_from_analysis(db, result.image_data.exif, image.id.clone())
            .await?;

        // Tags insertion
        tags::ActiveModel::create_from_analysis(db, result.image_data.tags, image.id.clone())
            .await?;

        // Weather insertion
        if let Some(weather_data) = result.image_data.weather {
            weather::ActiveModel::create_from_analysis(db, weather_data, image.id.clone()).await?;
        }

        // Frame processing
        visual_features::ActiveModel::create_from_analysis(
            db,
            &result.frame_data,
            image.id.clone(),
        )
        .await?;

        Ok(image)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// # Errors
    /// Returns `DbErr` if there is an error executing the database query.
    pub async fn get_relative_paths<C>(db: &C) -> Result<HashSet<String>, DbErr>
    where
        C: ConnectionTrait,
    {
        let paths: HashSet<String> = Self::find()
            .select_column(images::Column::RelativePath)
            .into_tuple()
            .all(db)
            .await?
            .into_iter()
            .collect();
        Ok(paths)
    }
}
