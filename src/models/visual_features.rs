pub use super::_entities::visual_features::{ActiveModel, Entity, Model};
use crate::api::analyze_structs::FrameDataOutput;
use crate::models::{face_boxes, object_boxes, ocr_boxes};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

pub type VisualFeatures = Entity;

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
    pub async fn create_from_analysis<C>(
        db: &C,
        frames: &Vec<FrameDataOutput>,
        image_id: String,
    ) -> Result<Vec<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut results: Vec<Model> = Vec::new();
        for (i, frame) in frames.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::cast_possible_truncation)]
            let frame_percentage = (i as f32 / frames.len() as f32 * 100.0) as i32;
            let vf = ActiveModel {
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
                image_id: Set(image_id.clone()),
                ..Default::default()
            }
            .insert(db)
            .await?;

            // OCR boxes
            ocr_boxes::ActiveModel::create_from_analysis(db, &frame.ocr.ocr_boxes, vf.id)
                .await?;

            // Todo: cluster faces

            // Face boxes
            face_boxes::ActiveModel::create_from_analysis(db, &frame.faces, vf.id)
                .await?;

            // Object boxes
            object_boxes::ActiveModel::create_from_analysis(db, &frame.objects, vf.id)
                .await?;

            // todo get full vf here
            results.push(vf);
        }
        Ok(results)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
