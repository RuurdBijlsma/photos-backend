use loco_rs::app::Hooks;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::api::analyze_api;
use photos_backend::app::App;
use photos_backend::common::settings::Settings;
use photos_backend::models::users::RegisterParams;
use photos_backend::models::{images, users};

// Helper: parse an ISO 8601 datetime string

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = playground::<App>().await?;
    let settings = Settings::from_context(&ctx);
    App::truncate(&ctx).await?;
    println!("Cleared DB!");
    Ok(())
}
