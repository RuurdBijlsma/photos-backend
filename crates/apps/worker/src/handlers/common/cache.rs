use color_eyre::Result;
use common_types::ml_analysis::PyVisualAnalysis;
use directories::ProjectDirs;
use generate_thumbnails::copy_dir_contents;
use media_analyzer::AnalyzeResult;
use std::path::{Path, PathBuf};
use tokio::fs;

const THUMBNAILS_FOLDER: &str = "thumbnails";
const INGEST_RESULT_FILENAME: &str = "ingest_result.json";
const ANALYSIS_RESULT_FILENAME: &str = "analysis_result.json";

pub fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_mmap_rayon(path)?;

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

fn cache_folder() -> PathBuf {
    ProjectDirs::from("dev", "ruurd", "photos").map_or_else(
        || Path::new(".cache").to_path_buf(),
        |proj| proj.cache_dir().to_path_buf(),
    )
}

async fn hash_cache_folder(hash: &str) -> Result<PathBuf> {
    let hash_folder = cache_folder().join(hash);
    if !hash_folder.exists() {
        fs::create_dir_all(&hash_folder).await?;
    }
    Ok(hash_folder)
}

pub async fn get_thumbnail_cache(hash: &str) -> Result<Option<PathBuf>> {
    let thumbs_folder = cache_folder().join(hash).join(THUMBNAILS_FOLDER);
    if !thumbs_folder.exists() {
        return Ok(None);
    }
    Ok(Some(thumbs_folder))
}

pub async fn write_thumbnail_cache(hash: &str, source_folder: &Path) -> Result<()> {
    let dest_folder = cache_folder().join(hash).join(THUMBNAILS_FOLDER);
    if !dest_folder.exists() {
        fs::create_dir_all(&dest_folder).await?;
    }
    copy_dir_contents(source_folder, &dest_folder).await?;
    Ok(())
}

pub async fn get_ingest_cache(hash: &str) -> Result<Option<AnalyzeResult>> {
    let process_cache_file = cache_folder().join(hash).join(INGEST_RESULT_FILENAME);
    if !process_cache_file.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(process_cache_file).await?;
    let analyze_result: AnalyzeResult = serde_json::from_str(&data)?;
    Ok(Some(analyze_result))
}

pub async fn write_ingest_cache(hash: &str, analyze_result: &AnalyzeResult) -> Result<()> {
    let hash_folder = hash_cache_folder(hash).await?;
    let ingest_cache_file = hash_folder.join(INGEST_RESULT_FILENAME);
    let json = serde_json::to_string(analyze_result)?;
    fs::write(ingest_cache_file, json).await?;
    Ok(())
}

pub async fn get_analysis_cache(hash: &str) -> Result<Option<Vec<PyVisualAnalysis>>> {
    let process_cache_file = cache_folder().join(hash).join(ANALYSIS_RESULT_FILENAME);
    if !process_cache_file.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(process_cache_file).await?;
    let analysis: Vec<PyVisualAnalysis> = serde_json::from_str(&data)?;
    Ok(Some(analysis))
}

pub async fn write_analysis_cache(hash: &str, analysis: &[PyVisualAnalysis]) -> Result<()> {
    let hash_folder = hash_cache_folder(hash).await?;
    let ingest_cache_file = hash_folder.join(ANALYSIS_RESULT_FILENAME);
    let json = serde_json::to_string(analysis)?;
    fs::write(ingest_cache_file, json).await?;
    Ok(())
}
