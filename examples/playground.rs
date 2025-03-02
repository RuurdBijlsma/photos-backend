#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::api::analyze_api;
use photos_backend::app::App;
use photos_backend::common::settings::Settings;
use photos_backend::models::images;
use sea_orm::ActiveValue::Set;
use serde_json::Value;
use std::path::Path;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;

    let settings = Settings::from_context(&ctx);
    let image_path = "2025-01-05-16-18-09-355.jpg";
    let result = analyze_api::process_media(image_path.to_string(), &settings.processing_api_url)
        .await
        .map_err(|e| loco_rs::Error::wrap(e))?;

    println!("{:?}", result);
    let Value::String(filename) = &result.image_data.exif.file["FileName"] else {
        return Err(Error::Message("No Filename in json".to_string()));
    };

    // let image = images::ActiveModel {
    //     filename: Set(filename.to_string()),
    //     relative_path: Set(image_path.to_string()),
    //     hash:
    //
    //     ..Default::default()
    // };

    Ok(())
}
