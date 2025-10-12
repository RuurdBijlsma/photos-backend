use sqlx::{FromRow, Type};

#[derive(FromRow, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Job {
    pub id: i64,
    pub relative_path: String,
    pub job_type: JobType,
    pub priority: i32,
    pub user_id: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub dependency_attempts: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "job_type", rename_all = "lowercase")]
pub enum JobType {
    Ingest,
    Remove,
    Analysis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Failed,
    Done,
    Cancelled,
}