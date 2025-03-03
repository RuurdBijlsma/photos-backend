use chrono::NaiveDateTime;
use loco_rs::app::Hooks;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::api::analyze_api;
use photos_backend::app::App;
use photos_backend::common::settings::Settings;
use photos_backend::models::users::RegisterParams;
use photos_backend::models::{gps, images, locations, users};
use sea_orm::ActiveValue::Set;
use serde_json::Value;

fn parse_iso_datetime(datetime_str: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    // Parse the ISO 8601 datetime string
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = playground::<App>().await?;
    let settings = Settings::from_context(&ctx);
    App::truncate(&ctx).await?;

    let user = users::Model::create_with_password(
        &ctx.db,
        &RegisterParams {
            email: "ruurd@bijlsma.dev".to_string(),
            name: "rute".to_string(),
            password: "asdf".to_string(),
        },
    )
    .await?;
    println!("{:#?}", user);

    let image_path = "PXL_20250105_102926142.jpg";
    let result = analyze_api::process_media(image_path.to_string(), &settings.processing_api_url)
        .await
        .map_err(|e| Error::wrap(e))?;

    let Value::String(filename) = &result.image_data.exif.file["FileName"] else {
        return Err(Error::Message("No Filename in json".to_string()));
    };
    let datetime_local =
        parse_iso_datetime(&result.image_data.time.datetime_local).map_err(Error::wrap)?;
    let datetime_utc = result
        .image_data
        .time
        .datetime_utc
        .and_then(|s| parse_iso_datetime(&s).ok());

    let txn = ctx.db.begin().await?;
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
        // Foreign keys at the bottom
        user_id: Set(user.id),
        ..Default::default()
    };
    let image = image_active.insert(&txn).await?;

    if let Some(gps_result) = result.image_data.gps {
        let location_model = locations::Model::find_or_create_location(
            &txn,
            gps_result.location.country,
            gps_result.location.province,
            gps_result.location.city,
            gps_result.latitude,
            gps_result.longitude,
        )
        .await?;

        // Create GPS record with the generated image ID
        let gps = gps::ActiveModel {
            latitude: Set(gps_result.latitude),
            longitude: Set(gps_result.longitude),
            altitude: Set(gps_result.altitude),
            location_id: Set(location_model.id),
            // Foreign keys at the bottom
            image_id: Set(image.id.clone()),
            ..Default::default()
        };

        // Insert GPS
        gps.insert(&txn).await?;
    } else {
        println!("Image has no gps!");
    }

    // Commit transaction
    txn.commit().await?;

    println!("{:?}", image);

    Ok(())
}
