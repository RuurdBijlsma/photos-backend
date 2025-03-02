use std::path::Path;

pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        // Convert to lowercase and then match against known extensions.
        let ext_lower = ext.to_ascii_lowercase();
        matches!(
            ext_lower.as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "heif" | "avif"
        )
    } else {
        false
    }
}

pub fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        let ext_lower = ext.to_ascii_lowercase();
        matches!(
            ext_lower.as_str(),
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm"
        )
    } else {
        false
    }
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
