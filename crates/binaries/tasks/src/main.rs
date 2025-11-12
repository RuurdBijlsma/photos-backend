use color_eyre::Result;
use common_photos::{JobType, enqueue_job, get_db_pool};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    // let twenty_four_hours = Duration::from_secs(12 * 60 * 60);
    // let mut interval = time::interval(twenty_four_hours);
    //
    // loop {
    //     // The first tick of `interval` happens immediately.
    //     interval.tick().await;
    //
    //     tokio::spawn(async {
    //         let result: Result<()> = async {
    let pool = get_db_pool().await?;
    enqueue_job(&pool, JobType::Scan, None, None, None).await?;
    enqueue_job(&pool, JobType::CleanDB, None, None, None).await?;
    enqueue_job(&pool, JobType::ClusterPhotos, None, None, None).await?;
    enqueue_job(&pool, JobType::ClusterFaces, None, None, None).await?;
    Ok(())
    //         }
    //         .await;
    //         if let Err(e) = result {
    //             error!("Schedule run failed: {}", e);
    //         }
    //     });
    // }
}
