use crate::process_file::process_file;
use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use photos_core::{get_db_pool, get_media_dir, get_thumbnail_options, get_thumbnails_dir};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

pub mod process_file;

enum WorkResult {
    Processed,
    QueueEmpty,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let analyzer = Arc::new(Mutex::new(MediaAnalyzer::builder().build().await?));
    let pool = get_db_pool().await?;

    loop {
        let analyzer = Arc::clone(&analyzer);
        let pool = pool.clone();
        let media_dir = get_media_dir();

        println!("Checking for work...");

        let task = tokio::spawn(async move {
            let mut guard = analyzer.lock().await;
            do_work(&media_dir, &mut guard, &pool).await
        });

        match task.await? {
            Ok(WorkResult::Processed) => {
                println!("Item processed. Checking for next item immediately.");
            }
            Ok(WorkResult::QueueEmpty) => {
                println!("Queue is empty. Waiting for 30 seconds.");
                time::sleep(Duration::from_secs(30)).await;
            }
            Err(e) => {
                eprintln!("Worker failed: {}. Retrying in 5 seconds.", e);
                time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn do_work(
    media_dir: &Path,
    analyzer: &mut MediaAnalyzer,
    pool: &PgPool,
) -> Result<WorkResult> {
    let mut tx = pool.begin().await?;

    // Atomically select and lock a job
    let relative_path: Option<String> = sqlx::query_scalar(
        "SELECT relative_path from process_queue ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED",
    )
        .fetch_optional(&mut *tx)
        .await?;

    let Some(path) = relative_path else {
        return Ok(WorkResult::QueueEmpty);
    };

    println!("Processing file: {}", &path);
    let file = media_dir.join(&path);

    let thumb_dir = get_thumbnails_dir();
    let thumb_config = get_thumbnail_options(&thumb_dir);

    // Process the file. If it fails, the transaction will be rolled back,
    // and the job will become available again after a short time.
    process_file(&file, &thumb_config, analyzer, &mut tx).await?;

    // If processing is successful, delete the job from the queue
    sqlx::query("DELETE FROM process_queue WHERE relative_path = $1")
        .bind(&path)
        .execute(&mut *tx)
        .await?;

    // Commit the transaction to release the lock and finalize the deletion
    tx.commit().await?;

    Ok(WorkResult::Processed)
}
