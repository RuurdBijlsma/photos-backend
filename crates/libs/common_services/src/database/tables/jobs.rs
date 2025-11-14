use serde_json::Value;
use sqlx::{FromRow, Type};

#[derive(FromRow, Debug)]
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
    Ingest,
    Remove,
    Analysis,
    Scan,
    CleanDB,
    ClusterFaces,
    ClusterPhotos,
    ImportAlbum,
    ImportAlbumItem,
}

impl JobType {
    #[must_use]
    pub const fn get_priority(&self, is_video: bool) -> i32 {
        match self {
            Self::Ingest => {
                if is_video {
                    55
                } else {
                    50
                }
            }
            Self::Analysis => {
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
            Self::ImportAlbum => 25,
            Self::ImportAlbumItem => 24,
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
