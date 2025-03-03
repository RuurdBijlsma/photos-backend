use chrono::NaiveDateTime;
use std::path::Path;

/// Parse string into iso datetime
/// # Errors
/// If string can't be parsed.
pub fn parse_iso_datetime(
    datetime_str: &str,
) -> loco_rs::Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S"))
}

#[must_use]
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            // Convert to lowercase and then match against known extensions.
            let ext_lower = ext.to_ascii_lowercase();
            matches!(
                ext_lower.as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "heif" | "avif"
            )
        })
}

#[must_use]
pub fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            // Convert to lowercase and then match against known extensions.
            let ext_lower = ext.to_ascii_lowercase();
            matches!(
                ext_lower.as_str(),
                "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm"
            )
        })
}

#[must_use]
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
