use sqlx::PgPool;
use std::path::Path;

pub async fn remove_file(file: &Path, pool: &PgPool) -> color_eyre::Result<()> {
    let _file = file;
    let _pool = pool;
    // 1. set flag to not show in UI
    // 2. enqueue remove task so it's removed
    // Idk if this is best method
    Ok(())
}
