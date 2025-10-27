use color_eyre::Result;
use sqlx::{Postgres, Transaction};

/// Retrieves the ID of a media item from the database given its relative path.
///
/// # Errors
///
/// This function will return an error if the database query fails, for example
/// if no media item with the given `relative_path` is found.
pub async fn get_media_item_id(
    tx: &mut Transaction<'_, Postgres>,
    relative_path: &str,
) -> Result<String> {
    let id = sqlx::query_scalar!(
        "SELECT id FROM media_item WHERE relative_path = $1",
        relative_path
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}
