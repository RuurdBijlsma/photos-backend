use color_eyre::Result;
use common_photos::{enqueue_system_job, get_db_pool, JobType};

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
    enqueue_system_job(&pool, JobType::Scan).await?;
    enqueue_system_job(&pool, JobType::CleanDB).await?;
    enqueue_system_job(&pool, JobType::Cluster).await?;
    Ok(())
    //         }
    //         .await;
    //         if let Err(e) = result {
    //             error!("Schedule run failed: {}", e);
    //         }
    //     });
    // }
}
