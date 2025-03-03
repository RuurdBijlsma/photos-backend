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
            email: "user@example.com".to_string(),
            name: "user".to_string(),
            password: "pw".to_string(),
        },
    )
    .await?;

    // Process the media file
    let image_path = "PXL_20250106_121218134.jpg";

    let result = analyze_api::process_media(image_path.to_string(), &settings.processing_api_url)
        .await
        .map_err(Error::wrap)?;

    let txn = ctx.db.begin().await?;

    let image = images::ActiveModel::create_from_analysis(&txn, &user, image_path, result).await?;

    txn.commit().await?;

    println!("Image record inserted with id: {:?}", image.id);
    Ok(())
}
