use app_state::{load_app_settings};
use color_eyre::Result;
use common_services::database::get_db_pool;
use common_services::database::jobs::JobType;
use common_services::job_queue::enqueue_job;

pub async fn run_tasks() -> Result<()> {
    // let twenty_four_hours = Duration::from_secs(12 * 60 * 60);
    // let mut interval = time::interval(twenty_four_hours);
    //
    // loop {
    //     // The first tick of `interval` happens immediately.
    //     interval.tick().await;
    //
    //     tokio::spawn(async {
    //         let result: Result<()> = async {
    let settings = load_app_settings()?;
    let pool = get_db_pool(&settings.secrets.database_url).await?;
    enqueue_job::<()>(&pool, &settings, JobType::Scan).call().await?;
    enqueue_job::<()>(&pool, &settings, JobType::CleanDB).call().await?;
    enqueue_job::<()>(&pool, &settings, JobType::ClusterPhotos)
        .call()
        .await?;
    enqueue_job::<()>(&pool, &settings, JobType::ClusterFaces)
        .call()
        .await?;
    Ok(())
    //         }
    //         .await;
    //         if let Err(e) = result {
    //             error!("Schedule run failed: {}", e);
    //         }
    //     });
    // }
}
