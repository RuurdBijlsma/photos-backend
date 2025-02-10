#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use photos_backend::{app::App, models::_entities::unique_faces};
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let ctx = playground::<App>().await?;

    let res = unique_faces::Entity::find().all(&ctx.db).await?;
    println!("{:?}", res);

    Ok(())
}
