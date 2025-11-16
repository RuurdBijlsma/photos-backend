use crate::handlers::{handle_create, handle_remove};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::PgPool;
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Runs the file system watcher.
///
/// This function sets up a channel to receive file system events and processes them
/// in a loop. Each event is handled in a separate asynchronous task.
pub async fn run(media_dir: &Path, pool: PgPool) -> notify::Result<()> {
    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.blocking_send(res) {
                error!("Failed to send event through channel: {}", e);
            }
        },
        Config::default(),
    )?;

    watcher.watch(media_dir, RecursiveMode::Recursive)?;
    info!("👁️ Watcher started on: {:?}", media_dir);

    while let Some(result) = rx.recv().await {
        let pool = pool.clone();
        tokio::spawn(async move {
            let event = match result {
                Ok(evt) => evt,
                Err(err) => {
                    error!("Watch error: {:?}", err);
                    return;
                }
            };
            process_event(event, &pool).await;
        });
    }

    Ok(())
}

/// Processes a single file system event from the watcher.
async fn process_event(event: Event, pool: &PgPool) {
    println!("{:?}", event.paths);
    let Some(path) = event.paths.first() else {
        return;
    };

    // Ignore temporary or hidden files.
    if path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|s| s.starts_with('.'))
    {
        info!("Ignoring hidden file event for: {:?}", path);
        return;
    }

    let result = match event.kind {
        EventKind::Create(_) => handle_create(path, pool).await,
        EventKind::Remove(_) => handle_remove(path, pool).await,
        _ => return,
    };

    if let Err(e) = result {
        warn!("Error handling file event for {:?}: {:?}", path, e);
    }
}
