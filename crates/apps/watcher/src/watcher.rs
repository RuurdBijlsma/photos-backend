use crate::handlers::{handle_create, handle_remove};
use app_state::{AppSettings, load_app_settings};
use color_eyre::eyre::Result;
use common_services::alert;
use common_services::database::get_db_pool;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub async fn start_watching(pool: PgPool,settings: AppSettings) -> Result<()> {
    if let Err(e) = run(&pool, &settings).await {
        alert!("Watcher failed with an error: {}", e);
    }

    Ok(())
}

/// Runs the file system watcher.
///
/// This function sets up a channel to receive file system events and processes them
/// in a loop. Each event is handled in a separate asynchronous task.
async fn run(pool: &PgPool, settings: &AppSettings) -> notify::Result<()> {
    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.blocking_send(res) {
                error!("Failed to send event through channel: {}", e);
            }
        },
        Config::default(),
    )?;

    watcher.watch(&settings.ingest.media_root, RecursiveMode::Recursive)?;
    info!("👁️ Watcher started on: {:?}", &settings.ingest.media_root);

    while let Some(result) = rx.recv().await {
        let pool = pool.clone();
        let settings = settings.clone();
        tokio::spawn(async move {
            let event = match result {
                Ok(evt) => evt,
                Err(err) => {
                    error!("Watch error: {:?}", err);
                    return;
                }
            };
            process_event(&pool, &settings, event).await;
        });
    }

    Ok(())
}

/// Processes a single file system event from the watcher.
async fn process_event(pool: &PgPool, settings: &AppSettings, event: Event) {
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
        EventKind::Create(_) => handle_create(pool, settings, path).await,
        EventKind::Remove(_) => handle_remove(pool, settings, path).await,
        _ => return,
    };

    if let Err(e) = result {
        warn!("Error handling file event for {:?}: {:?}", path, e);
    }
}
