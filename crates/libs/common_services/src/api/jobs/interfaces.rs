// File: crates/libs/common_services/src/api/jobs/interfaces.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::database::jobs::{JobStatus, JobType};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub id: i64,
    pub relative_path: Option<String>,
    pub user_id: Option<i32>,
    pub job_type: JobType,
    pub payload: Option<Value>,
    pub priority: i32,
    pub status: JobStatus,
    pub attempts: i32,
    pub dependency_attempts: i32,
    pub max_attempts: i32,
    pub owner: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub last_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobsResponse {
    pub running: Vec<JobInfo>,
    pub queued: Vec<JobInfo>,
    pub failed: Vec<JobInfo>,
    pub cancelled: Vec<JobInfo>,
    pub recently_done: Vec<JobInfo>,
}