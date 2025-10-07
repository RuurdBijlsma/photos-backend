use common_photos::{enqueue_file_ingest, enqueue_file_remove};
use futures::channel::mpsc::{Receiver, channel};
use futures::{SinkExt, StreamExt};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use sqlx::{Pool, Postgres};
use std::path::Path;
use tracing::{error, info, warn};

async fn handle_create_file(file: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    info!("File created {:?}", file);

    enqueue_file_ingest(file, pool).await?;

    Ok(())
}

async fn handle_remove_file(path: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    info!("File removed {:?}", path);

    enqueue_file_remove(path, pool).await?;

    Ok(())
}

pub fn start_watching(media_dir: &Path, pool: &Pool<Postgres>) -> color_eyre::Result<()> {
    futures::executor::block_on(async {
        if let Err(e) = async_watch(media_dir, pool).await {
            error!("async_watch error: {:?}", e)
        }
    });

    Ok(())
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch(media_dir: &Path, pool: &Pool<Postgres>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(media_dir.as_ref(), RecursiveMode::Recursive)?;

    while let Some(result) = rx.next().await {
        match result {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    if let Some(file) = event.paths.first() {
                        let result = handle_create_file(file, pool).await;
                        if let Err(e) = result {
                            warn!("Error handling file create: {:?}", e);
                        }
                    }
                }
                EventKind::Remove(_) => {
                    if let Some(file) = event.paths.first() {
                        let result = handle_remove_file(file, pool).await;
                        if let Err(e) = result {
                            warn!("Error handling file remove: {:?}", e);
                        }
                    }
                }
                _ => {}
            },
            Err(err) => error!("Watch error: {:?}", err),
        }
    }

    Ok(())
}
