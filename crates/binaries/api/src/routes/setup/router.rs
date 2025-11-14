use crate::api_state::ApiState;
use crate::setup::handlers::{
    get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder,
    post_start_processing,
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn setup_admin_router() -> Router<ApiState> {
    Router::new()
        .route("/setup/disk-info", get(get_disk_response))
        .route("/setup/media-sample", get(get_folder_media_sample))
        .route("/setup/unsupported-files", get(get_folder_unsupported))
        .route("/setup/folders", get(get_folders))
        .route("/setup/make-folder", post(make_folder))
        .route("/setup/start-processing", post(post_start_processing))
}
