use app_state::AppSettings;
use color_eyre::Result;
use common_services::database::jobs::JobType;
use common_services::job_queue::enqueue_job;
use sqlx::PgPool;

pub async fn run_tasks(pool: PgPool,settings: AppSettings) -> Result<()> {
    // let twenty_four_hours = Duration::from_secs(12 * 60 * 60);
    // let mut interval = time::interval(twenty_four_hours);
    //
    // loop {
    //     // The first tick of `interval` happens immediately.
    //     interval.tick().await;
    //
    //     tokio::spawn(async {
    //         let result: Result<()> = async {
    enqueue_job::<()>(&pool, &settings, JobType::Scan)
        .call()
        .await?;
    enqueue_job::<()>(&pool, &settings, JobType::CleanDB)
        .call()
        .await?;
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
