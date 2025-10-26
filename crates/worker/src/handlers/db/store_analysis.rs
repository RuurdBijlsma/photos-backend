use ml_analysis::VisualImageData;
use pgvector::Vector;
use sqlx::PgTransaction;

/// Stores the detailed results of a visual analysis for a media item in the database.
///
/// # Errors
///
/// This function will return an error if any of the database insertion queries fail
/// or if color histogram data cannot be serialized to JSON.
#[allow(clippy::too_many_lines)]
pub async fn store_visual_analysis(
    tx: &mut PgTransaction<'_>,
    media_item_id: &str,
    analyses: &[VisualImageData],
) -> color_eyre::Result<()> {
    for analysis in analyses {
        // Insert the main analysis record and get its new ID.
        let embed_vector = Vector::from(analysis.embedding.clone());
        let visual_analysis_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO visual_analysis (media_item_id, embedding)
            VALUES ($1, $2)
            RETURNING id
            "#,
            media_item_id,
            embed_vector as _,
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
            analysis.ocr.has_legible_text,
            analysis.ocr.ocr_text,
        )
        .fetch_one(&mut **tx)
        .await?;

        // --- OCR Box Data ---
        if let Some(ocr_boxes) = &analysis.ocr.ocr_boxes {
            for ocr_box in ocr_boxes {
                sqlx::query!(
                    r#"
                    INSERT INTO ocr_box (ocr_data_id, text, position_x, position_y, width, height, confidence)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    ocr_data_id,
                    ocr_box.text,
                    ocr_box.position.0,
                    ocr_box.position.1,
                    ocr_box.width,
                    ocr_box.height,
                    ocr_box.confidence,
                )
                    .execute(&mut **tx)
                    .await?;
            }
        }

        for face in &analysis.faces {
            let face_vector = Vector::from(face.embedding.clone());
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
                face.position.0,
                face.position.1,
                face.width,
                face.height,
                face.confidence,
                face.age,
                &face.sex,
                face.mouth_left.0,
                face.mouth_left.1,
                face.mouth_left.0,
                face.mouth_left.1,
                face.nose_tip.0,
                face.nose_tip.1,
                face.eye_left.0,
                face.eye_left.1,
                face.eye_right.0,
                face.eye_right.1,
                face_vector as _,
            )
                .execute(&mut **tx)
                .await?;
        }

        // --- Detected Objects ---
        for object in &analysis.objects {
            sqlx::query!(
                r#"
                INSERT INTO detected_object (visual_analysis_id, position_x, position_y, width, height, confidence, label)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                visual_analysis_id,
                object.position.0,
                object.position.1,
                object.width,
                object.height,
                object.confidence,
                &object.label,
            )
                .execute(&mut **tx)
                .await?;
        }

        // --- Quality Data (No changes needed) ---
        let quality = &analysis.quality_data;
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

        // --- Color Data (Updated for JSONB[] type) ---
        let color = &analysis.color_data;
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

        // --- Caption Data (No changes needed, already matches) ---
        let caption = &analysis.caption_data;
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
    }

    Ok(())
}
