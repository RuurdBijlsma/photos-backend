use serde_json::Value;
use sqlx::Type;

#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct Job {
    pub id: i64,
    pub payload: Option<Value>,
    pub relative_path: Option<String>,
    pub user_id: Option<i32>,
    pub job_type: JobType,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub dependency_attempts: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "job_type", rename_all = "snake_case")]
pub enum JobType {
    IngestMetadata,
    IngestThumbnails,
    IngestAnalysis,
    Remove,
    Scan,
    CleanDB,
    ClusterFaces,
    ClusterPhotos,
    ImportAlbumItem,
}

impl JobType {
    #[must_use]
    pub const fn get_priority(&self, is_video: bool) -> i32 {
        match self {
            Self::IngestMetadata => {
                50
            }
            Self::IngestThumbnails => {
                if is_video {
                    65
                } else {
                    60
                }
            }
            Self::IngestAnalysis => {
                if is_video {
                    95
                } else {
                    90
                }
            }
            Self::Remove => 0,
            Self::Scan => 10,
            Self::CleanDB => 20,
            Self::ClusterFaces => 30,
            Self::ClusterPhotos => 35,
            Self::ImportAlbumItem => 25,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Failed,
    Done,
    Cancelled,
}
