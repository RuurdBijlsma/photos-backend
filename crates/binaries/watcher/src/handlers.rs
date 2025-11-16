use common_services::alert;
use common_services::database::app_user::user_from_relative_path;
use common_services::database::jobs::JobType;
use common_services::get_settings::media_dir;
use common_services::job_queue::{enqueue_full_ingest, enqueue_job};
use common_services::utils::relative_path_abs;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;
use tracing::warn;
use walkdir::WalkDir;

async fn is_file_in_db(path: &Path, pool: &PgPool) -> color_eyre::Result<bool> {
    let relative_path = relative_path_abs(path)?;
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM media_item WHERE relative_path = $1
            UNION ALL
            SELECT 1 FROM jobs WHERE relative_path = $1
        )
        "#,
        relative_path
    )
    .fetch_one(pool)
    .await?;

    Ok(exists.unwrap_or(false))
}

pub async fn handle_create_event(path: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    if path.is_file() {
        handle_create_file(path, pool).await?;
    } else {
        handle_create_folder(path, pool).await?;
    }
    Ok(())
}

pub async fn handle_remove_event(path: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    if is_file_in_db(path, pool).await? {
        handle_remove_file(path, pool).await?;
    } else {
        handle_remove_folder(path, pool).await?;
    }
    Ok(())
}

/// Handles a file creation event by enqueueing the file for ingestion.
///
/// # Errors
///
/// * Returns an error if `enqueue_file_ingest` fails, typically due to a database issue.
async fn handle_create_file(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    info!("File created {:?}", file);

    let rel_path = relative_path_abs(file)?;
    if let Some(user) = user_from_relative_path(&rel_path, pool).await? {
        enqueue_full_ingest(pool, &rel_path, user.id).await?;
    } else {
        alert!("[Create file event] Cannot find user from relative path.");
    }

    Ok(())
}

/// Handles a file removal event by enqueueing the file for removal.
///
/// # Errors
///
/// * Returns an error if `enqueue_file_remove` fails, typically due to a database issue.
async fn handle_remove_file(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    info!("File removed {:?}", file);

    let rel_path = relative_path_abs(file)?;
    if let Some(user) = user_from_relative_path(&rel_path, pool).await? {
        enqueue_job::<()>(pool, JobType::Remove)
            .relative_path(&rel_path)
            .user_id(user.id)
            .call()
            .await?;
    } else {
        alert!("[Create file event] Cannot find user from relative path.");
    }

    Ok(())
}

async fn handle_create_folder(folder: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    println!("[handle_create_folder] {:?}", folder);
    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        println!("{:?}", entry);
        if entry.metadata()?.is_file() {
            handle_create_file(entry.path(), pool).await?
        }
    }

    Ok(())
}

async fn handle_remove_folder(folder: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    println!("[handle_remove_folder] {:?}", folder);
    let relative_dir = relative_path_abs(folder)?;

    let relative_paths = sqlx::query_scalar!(
        r"
        SELECT relative_path
        FROM media_item
        WHERE relative_path LIKE $1
    ",
        format!("{relative_dir}%")
    )
    .fetch_all(pool)
    .await?;

    for relative_path in relative_paths {
        handle_remove_file(&media_dir().join(relative_path), pool).await?;
    }

    Ok(())
}
