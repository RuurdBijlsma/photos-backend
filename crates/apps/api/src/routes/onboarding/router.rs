use crate::api_state::ApiState;
use crate::onboarding::handlers::{
    get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder,
    post_start_processing,
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn onboarding_admin_routes() -> Router<ApiState> {
    Router::new()
        .route("/onboarding/disk-info", get(get_disk_response))
        .route("/onboarding/media-sample", get(get_folder_media_sample))
        .route("/onboarding/unsupported-files", get(get_folder_unsupported))
        .route("/onboarding/folders", get(get_folders))
        .route("/onboarding/make-folder", post(make_folder))
        .route("/onboarding/start-processing", post(post_start_processing))
}
