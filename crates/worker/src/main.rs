use crate::queue_logic::process_one_job;
use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use common_photos::{get_db_pool, get_media_dir, worker_config};
use std::time::Duration;
use tokio::time;
use tracing::{info, warn};

mod db_helpers;
pub mod ingest_file;
mod queue_logic;
mod remove_file;

pub enum WorkResult {
    Processed,
    QueueEmpty,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    info!("[Worker PID: {}] Starting.", std::process::id());
    let mut analyzer = MediaAnalyzer::builder().build().await?;
    let media_dir = get_media_dir();
    let config = worker_config();
    let pool = get_db_pool().await?;
    let mut last_check_was_work = true;

    loop {
        let result = process_one_job(&media_dir, &mut analyzer, &pool).await;

        match result {
            Ok(WorkResult::Processed) => {
                last_check_was_work = true;
            }
            Ok(WorkResult::QueueEmpty) => {
                if last_check_was_work {
                    info!("No jobs, sleeping... 💤");
                }
                last_check_was_work = false;
                time::sleep(Duration::from_secs(config.wait_after_empty_queue_s)).await;
            }
            Err(e) => {
                last_check_was_work = true;
                warn!(
                    "Job failed: {}. Retrying in {}.",
                    e, config.wait_after_error_s
                );
                time::sleep(Duration::from_secs(config.wait_after_error_s)).await;
            }
        }
    }
}
