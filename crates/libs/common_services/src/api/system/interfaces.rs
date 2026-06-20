use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub disk_available: u64,
    pub disk_used: u64,
    pub disk_total: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiskStats {
    pub thumbnail_drive: DiskInfo,
    pub media_drive: DiskInfo,
    pub are_same_drive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub has_clustered_people: bool,
    pub has_clustered_photos: bool,
    pub disk: DiskStats,
}
