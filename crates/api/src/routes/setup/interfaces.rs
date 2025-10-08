use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct FolderQuery {
    pub folder: PathBuf,
}

#[derive(Deserialize)]
pub struct MakeFolderBody {
    pub base_folder: PathBuf,
    pub new_name: String,
}

#[derive(Serialize)]
pub struct PathInfoResponse {
    pub folder: String,
    pub disk_available: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub read_access: bool,
    pub write_access: bool,
}

#[derive(Serialize)]
pub struct MediaSampleResponse {
    pub read_access: bool,
    pub folder: String,
    pub photo_count: usize,
    pub video_count: usize,
    pub samples: Vec<String>,
}

#[derive(Serialize)]
pub struct UnsupportedFilesResponse {
    pub read_access: bool,
    pub folder: String,
    pub inaccessible_entries: Vec<String>,
    pub unsupported_files: HashMap<String, Vec<String>>,
    pub unsupported_count: usize,
}

#[derive(Serialize)]
pub struct DiskResponse {
    pub media_folder: PathInfoResponse,
    pub thumbnails_folder: PathInfoResponse,
}
