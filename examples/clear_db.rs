use loco_rs::app::Hooks;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = playground::<App>().await?;
    App::truncate(&ctx).await?;
    println!("Cleared DB!");
    Ok(())
}
