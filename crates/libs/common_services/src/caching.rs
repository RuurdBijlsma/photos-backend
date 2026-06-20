use std::path::{Path, PathBuf};
use directories::ProjectDirs;

pub fn hash_file(path: &Path) -> color_eyre::Result<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_mmap_rayon(path)?;
    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

pub fn cache_root() -> PathBuf {
    ProjectDirs::from("dev", "ruurd", "photos").map_or_else(
        || Path::new(".cache").to_path_buf(),
        |proj| proj.cache_dir().to_path_buf(),
    )
}