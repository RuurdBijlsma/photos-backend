use crate::indexer::process_file::process_file;
use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use media_analyzer::MediaAnalyzer;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use ruurd_photos_thumbnail_generation::ThumbOptions;
use sqlx::{Pool, Postgres};
use std::path::Path;

async fn handle_create_file(
    path: &Path,
    config: &ThumbOptions,
    analyzer: &mut MediaAnalyzer,
    pool: &Pool<Postgres>,
) -> color_eyre::Result<()> {
    println!("Created {:?}", path);

    process_file(path, config, analyzer, pool).await?;

    Ok(())
}

async fn handle_remove_file(
    path: &Path,
    config: &ThumbOptions,
    pool: &Pool<Postgres>,
) -> color_eyre::Result<()> {
    println!("Removed {:?}", path);

    Ok(())
}

pub fn start_watching(
    path: &Path,
    config: &ThumbOptions,
    pool: &Pool<Postgres>,
) -> color_eyre::Result<()> {
    futures::executor::block_on(async {
        let analyzer_result = MediaAnalyzer::builder().build().await;
        if let Ok(mut analyzer) = analyzer_result {
            if let Err(e) = async_watch(path, config, &mut analyzer, pool).await {
                println!("error: {:?}", e)
            }
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

async fn async_watch<P: AsRef<Path>>(
    path: P,
    config: &ThumbOptions,
    analyzer: &mut MediaAnalyzer,
    pool: &Pool<Postgres>,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(result) = rx.next().await {
        match result {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    if let Some(file) = event.paths.first() {
                        let result = handle_create_file(file, config, analyzer, pool).await;
                        if let Err(e) = result {
                            eprintln!("Error handling file create: {:?}", e);
                        }
                    }
                }
                EventKind::Remove(_) => {
                    if let Some(file) = event.paths.first() {
                        let result = handle_remove_file(file, config, pool).await;
                        if let Err(e) = result {
                            eprintln!("Error handling file remove: {:?}", e);
                        }
                    }
                }
                _ => {}
            },
            Err(err) => eprintln!("Watch error: {:?}", err),
        }
    }

    Ok(())
}
