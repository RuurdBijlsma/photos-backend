use media_analyzer::MediaAnalyzer;
use sqlx::PgTransaction;
use std::path::Path;
use tracing::info;

pub async fn ml_analyze_file(
    _file: &Path,
    _analyzer: &mut MediaAnalyzer,
    _tx: &mut PgTransaction<'_>,
) -> color_eyre::Result<()> {
    info!("Running ML analysis...");

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
