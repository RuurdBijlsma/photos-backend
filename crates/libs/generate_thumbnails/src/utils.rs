use color_eyre::eyre::Result;
use std::ffi::OsString;
use std::path::Path;
use tokio::fs;

/// Converts a `Path` to an `OsString` for use in command-line arguments.
pub fn path_to_os_string(p: &Path) -> OsString {
    p.as_os_str().to_owned()
}

/// Moves the contents of one directory to another.
pub async fn move_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).await?;
    let mut entries = fs::read_dir(src).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);
            fs::rename(entry.path(), dst_path).await?;
        }
    }

    Ok(())
}