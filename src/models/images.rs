pub use super::_entities::images::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::{FaceSex, MediaAnalyzerOutput};
use crate::models::_entities::images;
use crate::models::{
    face_boxes, gps, locations, metadata, object_boxes, ocr_boxes, tags, visual_features, weather,
};
use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use sea_orm::SelectColumns;
use std::collections::HashSet;
use std::path::Path;

pub type Images = Entity;

fn parse_iso_datetime(datetime_str: &str) -> loco_rs::Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S"))
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
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
        let image = Self {
            filename: Set(filename.to_string()),
            relative_path: Set(image_path.to_string()),
            width: Set(result.image_data.exif.width),
            height: Set(result.image_data.exif.height),
            duration: Set(result.image_data.exif.duration),
            format: Set(result.image_data.exif.format),
            size_bytes: Set(result.image_data.exif.size_bytes),
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
            let location = locations::Model::find_or_create_location(
                db,
                gps_data.location.country,
                gps_data.location.province,
                gps_data.location.city,
                gps_data.latitude,
                gps_data.longitude,
            )
            .await?;

            gps::ActiveModel {
                latitude: Set(gps_data.latitude),
                longitude: Set(gps_data.longitude),
                altitude: Set(gps_data.altitude),
                location_id: Set(location.id),
                image_id: Set(image.id.clone()),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        // Metadata insertion
        metadata::ActiveModel {
            exif_tool: Set(result.image_data.exif.exif_tool),
            file: Set(result.image_data.exif.file),
            composite: Set(result.image_data.exif.composite),
            exif: Set(result.image_data.exif.exif),
            xmp: Set(result.image_data.exif.xmp),
            mpf: Set(result.image_data.exif.mpf),
            jfif: Set(result.image_data.exif.jfif),
            icc_profile: Set(result.image_data.exif.icc_profile),
            gif: Set(result.image_data.exif.gif),
            png: Set(result.image_data.exif.png),
            quicktime: Set(result.image_data.exif.quicktime),
            matroska: Set(result.image_data.exif.matroska),
            image_id: Set(image.id.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // Tags insertion
        tags::ActiveModel {
            use_panorama_viewer: Set(result.image_data.tags.use_panorama_viewer),
            is_photosphere: Set(result.image_data.tags.is_photosphere),
            projection_type: Set(result.image_data.tags.projection_type),
            is_motion_photo: Set(result.image_data.tags.is_motion_photo),
            motion_photo_presentation_timestamp: Set(result
                .image_data
                .tags
                .motion_photo_presentation_timestamp),
            is_night_sight: Set(result.image_data.tags.is_night_sight),
            is_hdr: Set(result.image_data.tags.is_hdr),
            is_burst: Set(result.image_data.tags.is_burst),
            burst_id: Set(result.image_data.tags.burst_id),
            is_timelapse: Set(result.image_data.tags.is_timelapse),
            is_slowmotion: Set(result.image_data.tags.is_slowmotion),
            is_video: Set(result.image_data.tags.is_video),
            capture_fps: Set(result.image_data.tags.capture_fps),
            video_fps: Set(result.image_data.tags.video_fps),
            image_id: Set(image.id.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // Weather insertion
        if let Some(weather_data) = result.image_data.weather {
            let recorded_at = weather_data
                .weather_recorded_at
                .as_ref()
                .and_then(|ts| parse_iso_datetime(ts).ok());
            let weather_active = weather::ActiveModel {
                weather_recorded_at: Set(recorded_at),
                weather_temperature: Set(weather_data.weather_temperature),
                weather_dewpoint: Set(weather_data.weather_dewpoint),
                weather_relative_humidity: Set(weather_data.weather_relative_humidity),
                weather_precipitation: Set(weather_data.weather_precipitation),
                weather_wind_gust: Set(weather_data.weather_wind_gust),
                weather_pressure: Set(weather_data.weather_pressure),
                weather_sun_hours: Set(weather_data.weather_sun_hours),
                weather_condition: Set(weather_data.weather_condition.map(|c| c.to_string())),
                image_id: Set(image.id.clone()),
                ..Default::default()
            };
            weather_active.insert(db).await?;
        }

        // Frame processing
        for (i, frame) in result.frame_data.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::cast_possible_truncation)]
            let frame_percentage = (i as f32 / result.frame_data.len() as f32 * 100.0) as i32;
            let vf = visual_features::ActiveModel {
                frame_percentage: Set(frame_percentage),
                embedding: Set(frame.embedding.clone()),
                scene_type: Set(frame.classification.scene_type.clone()),
                people_type: Set(frame.classification.people_type.clone()),
                animal_type: Set(frame.classification.animal_type.clone()),
                document_type: Set(frame.classification.document_type.clone()),
                object_type: Set(frame.classification.object_type.clone()),
                activity_type: Set(frame.classification.activity_type.clone()),
                event_type: Set(frame.classification.event_type.clone()),
                weather_condition: Set(frame
                    .classification
                    .weather_condition
                    .map(|c| c.to_string())),
                is_outside: Set(frame.classification.is_outside),
                is_landscape: Set(frame.classification.is_landscape),
                is_cityscape: Set(frame.classification.is_cityscape),
                is_travel: Set(frame.classification.is_travel),
                has_legible_text: Set(frame.ocr.has_legible_text),
                ocr_text: Set(frame.ocr.ocr_text.clone()),
                document_summary: Set(frame.ocr.document_summary.clone()),
                measured_sharpness: Set(frame.measured_quality.measured_sharpness),
                measured_noise: Set(frame.measured_quality.measured_noise),
                measured_brightness: Set(frame.measured_quality.measured_brightness),
                measured_contrast: Set(frame.measured_quality.measured_contrast),
                measured_clipping: Set(frame.measured_quality.measured_clipping),
                measured_dynamic_range: Set(frame.measured_quality.measured_dynamic_range),
                quality_score: Set(frame.measured_quality.quality_score),
                summary: Set(frame.summary.clone()),
                caption: Set(frame.caption.clone()),
                image_id: Set(image.id.clone()),
                ..Default::default()
            }
            .insert(db)
            .await?;

            // OCR boxes
            for ocr_box in &frame.ocr.ocr_boxes {
                let ocr_box_active = ocr_boxes::ActiveModel {
                    // Convert fixed-size array into a Vec for the DB if necessary
                    position: Set(ocr_box.position.to_vec()),
                    width: Set(ocr_box.width),
                    height: Set(ocr_box.height),
                    confidence: Set(ocr_box.confidence),
                    text: Set(ocr_box.text.clone()),
                    visual_feature_id: Set(vf.id),
                    ..Default::default()
                };
                ocr_box_active.insert(db).await?;
            }

            // Todo: cluster faces

            // Face boxes
            for face in &frame.faces {
                let face_active = face_boxes::ActiveModel {
                    position: Set(face.position.to_vec()),
                    width: Set(face.width),
                    height: Set(face.height),
                    confidence: Set(face.confidence),
                    age: Set(face.age),
                    sex: Set(match face.sex {
                        FaceSex::Male => "M".to_string(),
                        FaceSex::Female => "F".to_string(),
                    }),
                    mouth_left: Set(face.mouth_left.to_vec()),
                    mouth_right: Set(face.mouth_right.to_vec()),
                    nose_tip: Set(face.nose_tip.to_vec()),
                    eye_left: Set(face.eye_left.to_vec()),
                    eye_right: Set(face.eye_right.to_vec()),
                    embedding: Set(face.embedding.clone()),
                    visual_feature_id: Set(vf.id),
                    ..Default::default()
                };
                face_active.insert(db).await?;
            }

            // Object boxes
            for object in &frame.objects {
                let object_active = object_boxes::ActiveModel {
                    position: Set(object.position.to_vec()),
                    width: Set(object.width),
                    height: Set(object.height),
                    label: Set(object.label.clone()),
                    confidence: Set(object.confidence),
                    visual_feature_id: Set(vf.id),
                    ..Default::default()
                };
                object_active.insert(db).await?;
            }
        }

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
