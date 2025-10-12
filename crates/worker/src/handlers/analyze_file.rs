use media_analyzer::MediaAnalyzer;
use sqlx::{Executor, PgTransaction, Postgres};
use std::path::Path;
use tracing::info;
use common_photos::Job;

pub async fn analyze_file<'c, E>(
    _executor: E,
    job: &Job,
) -> color_eyre::Result<()>
where
    E: Executor<'c, Database = Postgres>,
{
    info!("Running ML analysis... {:?}", &job);

    // todo check if prereqs are here:
    // * media item in db
    // * thumbnails exist

    // Python::attach(|py| -> Result<(), PyErr> {
    //     let now = Instant::now();
    //     let analyzer = VisualAnalyzer::new(py).unwrap();
    //     let elapsed = now.elapsed();
    //     println!("Make analyzer took {elapsed:?}");
    //     let now = Instant::now();
    //     let caption = analyzer.caption_image(file, None)?;
    //     let elapsed = now.elapsed();
    //     println!("caption_image took {elapsed:?}");
    //     println!("Caption for {} is {caption}", file.display());
    //
    //     Ok(())
    // })?;

    Ok(())
}
