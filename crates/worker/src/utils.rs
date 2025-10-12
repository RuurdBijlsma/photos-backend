use common_photos::nice_id;
use std::sync::LazyLock;

pub static WORKER_ID: LazyLock<String> = LazyLock::new(|| nice_id(8));
pub fn worker_id() -> &'static String {
    &WORKER_ID
}

// simple exponential backoff: 2^attempt * 10 seconds
pub fn backoff_seconds(attempts: i32) -> i64 {
    let secs = 10 * (2_i64.pow(attempts as u32));
    secs.min(3600) // cap at 1h
}
