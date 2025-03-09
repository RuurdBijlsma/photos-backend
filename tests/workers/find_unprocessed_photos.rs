use crate::helpers;
use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use photos_backend::{
    app::App,
    workers::find_unprocessed_images::{FindUnprocessedImagesWorker, WorkerArgs},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_run_find_unprocessed_photos_worker() {
    let boot = boot_test::<App>().await.unwrap();

    // Execute the worker ensuring that it operates in 'ForegroundBlocking' mode, which prevents the addition of your worker to the background
    assert!(
        FindUnprocessedImagesWorker::perform_later(&boot.app_context, WorkerArgs {})
            .await
            .is_ok()
    );
    // Include additional assert validations after the execution of the worker
    helpers::teardown(&boot.app_context.db).await;
}
