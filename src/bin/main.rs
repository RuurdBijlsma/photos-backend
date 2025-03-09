use loco_rs::cli;
use migration::Migrator;
use photos_backend::app::App;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App, Migrator>().await
}
