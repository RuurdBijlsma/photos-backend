use common_photos::{enqueue_file_ingest, enqueue_file_remove};
use futures::channel::mpsc::{Receiver, channel};
use futures::{SinkExt, StreamExt};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::{Pool, Postgres};
use std::path::Path;
use tracing::{error, info, warn};

/// Handles a file creation event by enqueueing the file for ingestion.
///
/// # Errors
///
/// * Returns an error if `enqueue_file_ingest` fails, typically due to a database issue.
async fn handle_create_file(file: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    info!("File created {:?}", file);

    enqueue_file_ingest(file, pool).await?;

    Ok(())
}

/// Handles a file removal event by enqueueing the file for removal.
///
/// # Errors
///
/// * Returns an error if `enqueue_file_remove` fails, typically due to a database issue.
async fn handle_remove_file(path: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    info!("File removed {:?}", path);

    enqueue_file_remove(path, pool).await?;

    Ok(())
}

/// Starts the file system watcher for the specified media directory in a blocking manner.
pub fn start_watching(media_dir: &Path, pool: &Pool<Postgres>) {
    futures::executor::block_on(async {
        if let Err(e) = async_watch(media_dir, pool).await {
            error!("async_watch error: {:?}", e);
        }
    });
}

/// Creates a new recommended file system watcher and a channel to receive events.
///
/// # Errors
///
/// * Returns a `notify::Error` if the watcher cannot be initialized.
///
/// # Panics
///
/// * Panics if the created channel is closed and cannot receive events.
fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            });
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

/// Watches a directory asynchronously for file changes and dispatches events to handlers.
///
/// # Errors
///
/// * Returns a `notify::Error` if the watcher fails to start watching the specified path.
async fn async_watch(media_dir: &Path, pool: &Pool<Postgres>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(media_dir.as_ref(), RecursiveMode::Recursive)?;

    // The main loop is now much simpler, delegating all logic to the handler.
    while let Some(result) = rx.next().await {
        process_event(result, pool).await;
    }

    Ok(())
}

/// Processes a single file system event from the watcher.
async fn process_event(event_result: notify::Result<Event>, pool: &Pool<Postgres>) {
    let event = match event_result {
        Ok(evt) => evt,
        Err(err) => {
            error!("Watch error: {:?}", err);
            return;
        }
    };

    let Some(path) = event.paths.first() else {
        return;
    };

    let result = match event.kind {
        EventKind::Create(_) => handle_create_file(path, pool).await,
        EventKind::Remove(_) => handle_remove_file(path, pool).await,
        _ => return,
    };

    if let Err(e) = result {
        warn!("Error handling file event for {:?}: {:?}", path, e);
    }
}
