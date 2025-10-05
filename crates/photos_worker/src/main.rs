use crate::process_file::process_file;
use color_eyre::Result;
use media_analyzer::MediaAnalyzer;
use photos_core::{
    get_db_pool, get_media_dir, get_thumbnail_options, get_thumbnails_dir,
    max_worker_processing_retries,
};
use sqlx::{FromRow, PgPool};
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

#[derive(FromRow)]
struct Job {
    relative_path: String,
    retry_count: Option<i32>,
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
                println!("Item processed.");
            }
            Ok(WorkResult::QueueEmpty) => {
                println!("Queue is empty...");
                time::sleep(Duration::from_secs(30)).await;
            }
            Err(e) => {
                // This now only catches critical errors like a lost database connection
                eprintln!(
                    "A critical worker error occurred: {}. Retrying in 5 seconds.",
                    e
                );
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

    // Atomically select and lock a job, fetching all necessary columns
    let job: Option<Job> = sqlx::query_as(
        "SELECT relative_path, retry_count FROM process_queue ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED",
    )
        .fetch_optional(&mut *tx)
        .await?;

    // Use `let Some(...) else` for cleaner exit if queue is empty
    let Some(job) = job else {
        // No job found, transaction will be implicitly rolled back.
        return Ok(WorkResult::QueueEmpty);
    };

    let path = &job.relative_path;
    println!("Processing file: {}", path);
    let file = media_dir.join(path);

    let thumb_dir = get_thumbnails_dir();
    let thumb_config = get_thumbnail_options(&thumb_dir);

    // Attempt to process the file and match on the result
    let processing_result = process_file(&file, &thumb_config, analyzer, &mut tx).await;

    match processing_result {
        Ok(_) => {
            // SUCCESS: Delete the job from the queue
            sqlx::query("DELETE FROM process_queue WHERE relative_path = $1")
                .bind(job.relative_path)
                .execute(&mut *tx)
                .await?;
        }
        Err(e) => {
            // FAILURE: Decide whether to retry or move to dead-letter queue
            eprintln!("Error processing file '{}': {}", path, e);
            let current_retries = job.retry_count.unwrap_or(0);

            if current_retries >= max_worker_processing_retries() - 1 {
                // Max retries reached: move to failures queue
                println!(
                    "File has failed {} times. Moving to failures queue.",
                    current_retries
                );
                // TODO alert here

                // Insert into failures, ignoring if it's somehow already there
                sqlx::query("INSERT INTO queue_failures (relative_path) VALUES ($1) ON CONFLICT (relative_path) DO NOTHING")
                    .bind(path)
                    .execute(&mut *tx)
                    .await?;

                // Delete from the main queue
                sqlx::query("DELETE FROM process_queue WHERE relative_path = $1")
                    .bind(job.relative_path)
                    .execute(&mut *tx)
                    .await?;
            } else {
                // Increment retry count
                println!("Incrementing retry count for file.");
                sqlx::query("UPDATE process_queue SET retry_count = $1 WHERE relative_path = $2")
                    .bind(current_retries + 1)
                    .bind(job.relative_path)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }

    // Commit the transaction to finalize either the success or failure handling
    tx.commit().await?;

    // We return "Processed" even on failure because the queue item itself
    // was handled (by being retried or moved). This tells the main loop
    // to immediately look for the next job.
    Ok(WorkResult::Processed)
}
