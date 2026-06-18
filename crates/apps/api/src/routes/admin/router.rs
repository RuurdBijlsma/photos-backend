use crate::api_state::ApiContext;
use axum::{
    Router,
    routing::{get, post},
};
use crate::admin::handlers::{get_disk_response, get_folder_media_sample, get_folder_unsupported, get_folders, make_folder, post_start_processing};

pub fn admin_admin_routes() -> Router<ApiContext> {
    Router::new()
        .route("/admin/disk-info", get(get_disk_response))
        .route("/admin/media-sample", get(get_folder_media_sample))
        .route("/admin/unsupported-files", get(get_folder_unsupported))
        .route("/admin/folders", get(get_folders))
        .route("/admin/make-folder", post(make_folder))
        .route("/admin/start-processing", post(post_start_processing))
}
