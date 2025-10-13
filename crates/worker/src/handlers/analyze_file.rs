use color_eyre::eyre::eyre;
use common_photos::{Job, alert, file_is_ingested, media_dir};
use sqlx::{Executor, Postgres};
use tracing::warn;

pub async fn analyze_file<'c, E>(executor: E, job: &Job) -> color_eyre::Result<()>
where
    E: Executor<'c, Database = Postgres>,
{
    let file = media_dir().join(&job.relative_path);
    if !file_is_ingested(&file, executor).await? {
        alert!("Analysis job picked up while file is not properly ingested");
        return Err(eyre!("File is not properly ingested"));
    }

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
