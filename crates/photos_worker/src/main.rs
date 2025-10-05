use crate::queue_logic::process_one_job;
use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use photos_core::{
    get_db_pool, get_media_dir, worker_config,
};
use std::time::Duration;
use tokio::time;

pub mod process_file;
mod queue_logic;
mod db_helpers;

pub enum WorkResult {
    Processed,
    QueueEmpty,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("[Worker PID: {}] Starting.", std::process::id());
    let mut analyzer = MediaAnalyzer::builder().build().await?;
    let media_dir = get_media_dir();
    let config = worker_config();
    let pool = get_db_pool().await?;

    loop {
        let result = process_one_job(&media_dir, &mut analyzer, &pool).await;

        match result {
            Ok(WorkResult::Processed) => { /* no-op, just loop again */ }
            Ok(WorkResult::QueueEmpty) => {
                time::sleep(Duration::from_secs(config.wait_after_empty_queue_s)).await;
            }
            Err(e) => {
                eprintln!(
                    "A critical error occurred: {}. Retrying in {}.",
                    e, config.wait_after_error_s
                );
                time::sleep(Duration::from_secs(config.wait_after_error_s)).await;
            }
        }
    }
}
