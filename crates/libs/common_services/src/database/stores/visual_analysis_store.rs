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
            INSERT INTO quality (
                visual_analysis_id,
                exposure,
                contrast,
                sharpness,
                color_accuracy,
                composition,
                subject_clarity,
                visual_impact,
                creativity,
                color_harmony,
                storytelling,
                style_suitability,
                weighted_score,
                measured_blurriness,
                measured_noisiness,
                measured_exposure,
                measured_weighted_score
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        "#,
            visual_analysis_id,
            i16::from(quality.exposure),
            i16::from(quality.contrast),
            i16::from(quality.sharpness),
            i16::from(quality.color_accuracy),
            i16::from(quality.composition),
            i16::from(quality.subject_clarity),
            i16::from(quality.visual_impact),
            i16::from(quality.creativity),
            i16::from(quality.color_harmony),
            i16::from(quality.storytelling),
            i16::from(quality.style_suitability),
            quality.weighted_score,
            quality.measured_blurriness,
            quality.measured_noisiness,
            quality.measured_exposure,
            quality.measured_weighted_score,
        )
        .execute(&mut **tx)
        .await?;

        // --- Color Data ---
        let color = &analysis.colors;
        let themes_values: Vec<serde_json::Value> = color.themes.clone();
        let histogram_json = serde_json::to_value(&color.histogram)?;

        sqlx::query!(
            r#"
            INSERT INTO color (
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
        let caption = &analysis.classification;
        sqlx::query!(
            r#"
            INSERT INTO caption (
                visual_analysis_id, default_caption, main_subject, contains_pets, contains_vehicle,
                contains_landmarks, contains_people, contains_animals, contains_text, is_indoor, is_food, is_drink,
                is_event, is_document, is_landscape, is_cityscape, is_activity, setting,
                animal_type, food_name, drink_name, vehicle_type, event_type, landmark_name, ocr_text,
                document_type, people_count, people_mood, photo_type, activity_description
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30
            )
            "#,
            visual_analysis_id,
            caption.caption,
            caption.main_subject,
            caption.contains_pets,
            caption.contains_vehicle,
            caption.contains_landmarks,
            caption.contains_people,
            caption.contains_animals,
            caption.contains_text,
            caption.is_indoor,
            caption.is_food,
            caption.is_drink,
            caption.is_event,
            caption.is_document,
            caption.is_landscape,
            caption.is_cityscape,
            caption.is_activity,
            caption.setting,
            caption.animal_type,
            caption.food_name,
            caption.drink_name,
            caption.vehicle_type,
            caption.event_type,
            caption.landmark_name,
            caption.ocr_text,
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
