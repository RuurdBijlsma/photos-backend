use std::path::Path;

/// Converts a path to a POSIX-style string, replacing backslashes with forward slashes.
#[must_use]
pub fn to_posix_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}