use chrono::NaiveDateTime;
use loco_rs::app::Hooks;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::api::analyze_api;
use photos_backend::api::analyze_structs::FaceSex;
use photos_backend::app::App;
use photos_backend::common::settings::Settings;
use photos_backend::models::users::RegisterParams;
use photos_backend::models::{
    face_boxes, metadata, object_boxes, ocr_boxes, tags, visual_features, weather,
};
use photos_backend::models::{gps, images, locations, users};
use sea_orm::ActiveValue::Set;
use serde_json::Value;

// Helper: parse an ISO 8601 datetime string
fn parse_iso_datetime(datetime_str: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S"))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup context, settings and truncate database state for a fresh run
    let ctx = playground::<App>().await?;
    let settings = Settings::from_context(&ctx);
    App::truncate(&ctx).await?;

    // Create a user
    let user = users::Model::create_with_password(
        &ctx.db,
        &RegisterParams {
            email: "ruurd@bijlsma.dev".to_string(),
            name: "rute".to_string(),
            password: "asdf".to_string(),
        },
    )
    .await?;

    // Process the media file
    let image_path = "PXL_20250105_102926142.jpg";
    let result = analyze_api::process_media(image_path.to_string(), &settings.processing_api_url)
        .await
        .map_err(|e| Error::wrap(e))?;

    // Retrieve filename from exif data (ensuring it's a string)
    let Value::String(filename) = &result.image_data.exif.file["FileName"] else {
        return Err(Error::Message("No Filename in json".to_string()));
    };

    // Parse the provided datetime strings
    let datetime_local =
        parse_iso_datetime(&result.image_data.time.datetime_local).map_err(Error::wrap)?;
    let datetime_utc = result
        .image_data
        .time
        .datetime_utc
        .and_then(|s| parse_iso_datetime(&s).ok());

    // Begin a database transaction
    let txn = ctx.db.begin().await?;

    // Insert the main image record
    let image_active = images::ActiveModel {
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
        // Foreign key: owner
        user_id: Set(user.id),
        ..Default::default()
    };
    let image = image_active.insert(&txn).await?;

    // If GPS information is available, insert location and GPS records.
    if let Some(gps_result) = &result.image_data.gps {
        let location_model = locations::Model::find_or_create_location(
            &txn,
            gps_result.location.country.clone(),
            gps_result.location.province.clone(),
            gps_result.location.city.clone(),
            gps_result.latitude,
            gps_result.longitude,
        )
        .await?;

        let gps_active = gps::ActiveModel {
            latitude: Set(gps_result.latitude),
            longitude: Set(gps_result.longitude),
            altitude: Set(gps_result.altitude),
            location_id: Set(location_model.id),
            // Foreign key: image
            image_id: Set(image.id.clone()),
            ..Default::default()
        };
        gps_active.insert(&txn).await?;
    } else {
        println!("Image has no GPS data!");
    }

    // Insert metadata from the exif data into the metadata table.
    let metadata_active = metadata::ActiveModel {
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
    };
    metadata_active.insert(&txn).await?;

    // Insert tags data
    let tags_active = tags::ActiveModel {
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
    };
    tags_active.insert(&txn).await?;

    // Insert weather data if available
    if let Some(weather_data) = &result.image_data.weather {
        let weather_recorded_at = weather_data
            .weather_recorded_at
            .as_ref()
            .and_then(|ts| parse_iso_datetime(ts).ok());
        let weather_active = weather::ActiveModel {
            weather_recorded_at: Set(weather_recorded_at),
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
        weather_active.insert(&txn).await?;
    }

    // Process each frame and insert visual features along with OCR, faces and objects.
    for (i, frame) in result.frame_data.iter().enumerate() {
        // Calculate a frame percentage. Adjust the calculation as needed.
        let frame_percentage = ((i as f32 / result.frame_data.len() as f32) * 100.0) as i32;
        let vf_active = visual_features::ActiveModel {
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
        };
        let vf = vf_active.insert(&txn).await?;

        // Insert each OCR box
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
            ocr_box_active.insert(&txn).await?;
        }

        // Todo: cluster faces

        // Insert face boxes for each detected face.
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
            face_active.insert(&txn).await?;
        }

        // Insert object boxes for each detected object.
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
            object_active.insert(&txn).await?;
        }
    }

    // Commit the transaction once all inserts have succeeded.
    txn.commit().await?;

    println!("Image record inserted with id: {:?}", image.id);

    Ok(())
}
