use std::path::Path;
use tokio::fs;

pub async fn move_dir_contents(src: &Path, dst: &Path) -> color_eyre::Result<()> {
    fs::create_dir_all(dst).await?;
    let mut entries = fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if file_type.is_file() {
            fs::rename(entry.path(), dst_path).await?;
        }
    }

    Ok(())
}
