use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::PgPool;
use std::path::Path;
use tracing::{error, info, warn};
use crate::handlers::{handle_create_event, handle_remove_event};

/// Starts the file system watcher for the specified media directory in a blocking manner.
pub fn start_watching(media_dir: &Path, pool: &PgPool) {
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
async fn async_watch(media_dir: &Path, pool: &PgPool) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(media_dir.as_ref(), RecursiveMode::Recursive)?;

    // The main loop is now much simpler, delegating all logic to the handler.
    while let Some(result) = rx.next().await {
        process_event(result, pool).await;
    }

    Ok(())
}

/// Processes a single file system event from the watcher.
async fn process_event(event_result: notify::Result<Event>, pool: &PgPool) {
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

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
        && file_name.starts_with('.')
        && file_name.contains(".tmp")
    {
        info!("Ignoring temporary file event for: {:?}", path);
        return;
    }

    let result = match event.kind {
        EventKind::Create(_) => handle_create_event(path, pool).await,
        EventKind::Remove(_) => handle_remove_event(path, pool).await,
        _ => return,
    };
    if let Err(e) = result {
        warn!("Error handling file event for {:?}: {:?}", path, e);
    }
}
