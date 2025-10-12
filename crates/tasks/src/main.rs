mod clean_db;
pub mod scan_all;

use crate::clean_db::clean_db;
use crate::scan_all::run_scan;
use color_eyre::Result;
use common_photos::{get_db_pool};
use std::time::Duration;
use tokio::time;
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let twenty_four_hours = Duration::from_secs(24 * 60 * 60);
    let mut interval = time::interval(twenty_four_hours);

    loop {
        // The first tick of `interval` happens immediately.
        interval.tick().await;

        tokio::spawn(async {
            let result: Result<()> = async {
                let pool = get_db_pool().await?;
                if let Err(e) = run_scan(&pool).await {
                    // todo alert here
                    error!("Scanning failed: {}", e);
                }
                if let Err(e) = clean_db(&pool).await {
                    // todo alert here
                    error!("Clean db failed: {}", e);
                }

                Ok(())
            }
            .await;
            if let Err(e) = result {
                error!("Schedule run failed: {}", e);
            }
        });
    }
}
