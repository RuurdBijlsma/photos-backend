use crate::database::visual_analysis::visual_analysis::CreateVisualAnalysis;
use crate::database::DbError;
use sqlx::PgTransaction;

pub struct VisualAnalysisStore;

impl VisualAnalysisStore {
    /// Stores the detailed results of a visual analysis for a media item in the database.
    ///
    /// This function takes a single `VisualImageData` object and persists it, including all nested
    /// data like faces, objects, and color information. It returns the ID of the newly created
    /// `visual_analysis` record.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the database insertion queries fail
    /// or if the color histogram data cannot be serialized to JSON.
    #[allow(clippy::too_many_lines)]
    pub async fn create(
        tx: &mut PgTransaction<'_>,
        media_item_id: &str,
        analysis: &CreateVisualAnalysis,
    ) -> Result<i64, DbError> {
        // Insert the main analysis record and get its new ID.
        let visual_analysis_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO visual_analysis (media_item_id, embedding, percentage)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            media_item_id,
            analysis.embedding as _,
            analysis.percentage,
        )
        .fetch_one(&mut **tx)
        .await?;

        // --- OCR Data ---
        let ocr_data_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO ocr_data (visual_analysis_id, has_legible_text, ocr_text)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            visual_analysis_id,
            analysis.ocr_data.has_legible_text,
            analysis.ocr_data.ocr_text,
        )
        .fetch_one(&mut **tx)
        .await?;

        // --- OCR Box Data ---
        for ocr_box in &analysis.ocr_data.boxes {
            sqlx::query!(
                    r#"
                    INSERT INTO ocr_box (ocr_data_id, text, position_x, position_y, width, height, confidence)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    ocr_data_id,
                    ocr_box.text,
                    ocr_box.position_x,
                    ocr_box.position_y,
                    ocr_box.width,
                    ocr_box.height,
                    ocr_box.confidence,
                )
                    .execute(&mut **tx)
                    .await?;
        }

        // --- Face Data ---
        for face in &analysis.faces {
            sqlx::query!(
                r#"
                INSERT INTO face (
                    visual_analysis_id, position_x, position_y, width, height, confidence, age, sex,
                    mouth_left_x, mouth_left_y, mouth_right_x, mouth_right_y,
                    nose_tip_x, nose_tip_y, eye_left_x, eye_left_y, eye_right_x, eye_right_y,
                    embedding
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                "#,
                visual_analysis_id,
                face.position_x,
                face.position_y,
                face.width,
                face.height,
                face.confidence,
                face.age,
                &face.sex,
                face.mouth_left_x,
                face.mouth_left_y,
                face.mouth_right_x,
                face.mouth_right_y,
                face.nose_tip_x,
                face.nose_tip_y,
                face.eye_left_x,
                face.eye_left_y,
                face.eye_right_x,
                face.eye_right_y,
                face.embedding as _,
            )
                .execute(&mut **tx)
                .await?;
        }

        // --- Detected Objects ---
        for object in &analysis.detected_objects {
            sqlx::query!(
                r#"
                INSERT INTO detected_object (visual_analysis_id, position_x, position_y, width, height, confidence, label)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                visual_analysis_id,
                object.position_x,
                object.position_y,
                object.width,
                object.height,
                object.confidence,
                &object.label,
            )
                .execute(&mut **tx)
                .await?;
        }

        // --- Quality Data ---
        let quality = &analysis.quality;
        sqlx::query!(
            r#"
            INSERT INTO quality_data (visual_analysis_id, blurriness, noisiness, exposure, quality_score)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            visual_analysis_id,
            quality.blurriness,
            quality.noisiness,
            quality.exposure,
            quality.quality_score,
        )
            .execute(&mut **tx)
            .await?;

        // --- Color Data ---
        let color = &analysis.colors;
        let themes_values: Vec<serde_json::Value> = color.themes.clone();
        let histogram_json = serde_json::to_value(&color.histogram)?;

        sqlx::query!(
            r#"
            INSERT INTO color_data (
                visual_analysis_id, themes, prominent_colors,
                average_hue, average_saturation, average_lightness, histogram
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            visual_analysis_id,
            &themes_values,
            &color.prominent_colors,
            color.average_hue,
            color.average_saturation,
            color.average_lightness,
            histogram_json,
        )
        .execute(&mut **tx)
        .await?;

        // --- Caption Data ---
        let caption = &analysis.caption;
        sqlx::query!(
            r#"
            INSERT INTO caption_data (
                visual_analysis_id, default_caption, main_subject, contains_pets, contains_vehicle,
                contains_landmarks, contains_people, contains_animals, is_indoor, is_food_or_drink,
                is_event, is_document, is_landscape, is_cityscape, is_activity, setting, pet_type,
                animal_type, food_or_drink_type, vehicle_type, event_type, landmark_name,
                document_type, people_count, people_mood, photo_type, activity_description
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, $25, $26, $27
            )
            "#,
            visual_analysis_id,
            caption.default_caption,
            caption.main_subject,
            caption.contains_pets,
            caption.contains_vehicle,
            caption.contains_landmarks,
            caption.contains_people,
            caption.contains_animals,
            caption.is_indoor,
            caption.is_food_or_drink,
            caption.is_event,
            caption.is_document,
            caption.is_landscape,
            caption.is_cityscape,
            caption.is_activity,
            caption.setting,
            caption.pet_type,
            caption.animal_type,
            caption.food_or_drink_type,
            caption.vehicle_type,
            caption.event_type,
            caption.landmark_name,
            caption.document_type,
            caption.people_count,
            caption.people_mood,
            caption.photo_type,
            caption.activity_description,
        )
        .execute(&mut **tx)
        .await?;

        Ok(visual_analysis_id)
    }
}
