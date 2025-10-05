use crate::insert_query;
use crate::utils::get_relative_path_str;
use sqlx::PgPool;
use std::path::Path;

pub async fn enqueue_file(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let relative_path_str = get_relative_path_str(file)?;
    let mut tx = pool.begin().await?;

    println!("Enqueueing file: {:?}", file);
    insert_query!(&mut tx, "process_queue", {
        relative_path: relative_path_str,
    })
    .await?;

    tx.commit().await?;

    Ok(())
}
