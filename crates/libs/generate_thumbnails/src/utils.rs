use color_eyre::eyre::Result;
use std::ffi::OsString;
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

/// Converts a `Path` to an `OsString` for use in command-line arguments.
pub fn path_to_os_string(p: &Path) -> OsString {
    p.as_os_str().to_owned()
}

/// Copies the contents of one directory to another using the `walkdir` crate.
async fn copy_dir_with_walkdir(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let relative_path = src_path.strip_prefix(src)?;
        let dst_path = dst.join(relative_path);

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
    }
    Ok(())
}

/// Copies the contents of one directory to another and then deletes the source directory.
pub async fn move_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).await?;
    copy_dir_with_walkdir(src, dst).await?;
    fs::remove_dir_all(src).await?;

    Ok(())
}
