use chrono::NaiveDateTime;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::api::analyze_api;
use photos_backend::app::App;
use photos_backend::common::settings::Settings;
use photos_backend::models::users::RegisterParams;
use photos_backend::models::{images, users};
use sea_orm::ActiveValue::Set;
use serde_json::Value;
use std::path::Path;

fn parse_iso_datetime(datetime_str: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    // Parse the ISO 8601 datetime string
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S"))
}

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;
    let settings = Settings::from_context(&ctx);

    // let user = users::Model::create_with_password(
    //     &ctx.db,
    //     &RegisterParams {
    //         email: "ruurd@bijlsma.dev".to_string(),
    //         name: "rute".to_string(),
    //         password: "asdf".to_string(),
    //     },
    // )
    // .await?;
    // println!("{:#?}", user);

    let image_path = "2025-01-05-16-18-09-355.jpg";
    let result = analyze_api::process_media(image_path.to_string(), &settings.processing_api_url)
        .await
        .map_err(|e| loco_rs::Error::wrap(e))?;

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

    let image = images::ActiveModel {
        filename: Set(filename.to_string()),
        relative_path: Set(image_path.to_string()),
        width: Set(result.image_data.exif.width),
        height: Set(result.image_data.exif.height),
        duration: Set(result.image_data.exif.duration.map(|d| d as f32)),
        format: Set(result.image_data.exif.format),
        size_bytes: Set(result.image_data.exif.size_bytes),
        datetime_local: Set(datetime_local),
        datetime_utc: Set(datetime_utc),
        datetime_source: Set(result.image_data.time.datetime_source),
        timezone_name: Set(result.image_data.time.timezone_name),
        timezone_offset: Set(result.image_data.time.timezone_offset),
        user_id: Set(1),
        ..Default::default()
    };

    let image: images::Model = image.insert(&ctx.db).await?;
    println!("{:?}", image);

    Ok(())
}
