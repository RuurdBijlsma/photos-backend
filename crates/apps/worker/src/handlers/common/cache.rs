use color_eyre::Result;
use color_eyre::eyre::bail;
use common_types::ml_analysis::{MLChatAnalysis, MLFastAnalysis};
use directories::ProjectDirs;
use generate_thumbnails::copy_dir_contents;
use media_analyzer::MediaMetadata;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::warn;

// Category folder names
const THUMBNAILS_DIR: &str = "thumbnails";
const INGEST_DIR: &str = "ingest";
const ANALYSIS_DIR: &str = "analysis";
const LLM_DIR: &str = "llm";

const INGEST_CACHE_VERSION: u32 = 2;
const ANALYSIS_CACHE_VERSION: u32 = 1;
const LLM_CACHE_VERSION: u32 = 1;
const EXPECTED_EMBEDDING_LENGTH: usize = 768;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedIngestResult {
    pub media_metadata: MediaMetadata,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedAnalysisResult {
    pub fast_analyses: Vec<MLFastAnalysis>,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedLlmResult {
    pub llm_analyses: Vec<MLChatAnalysis>,
    pub version: u32,
}

pub fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_mmap_rayon(path)?;
    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

fn cache_root() -> PathBuf {
    ProjectDirs::from("dev", "ruurd", "photos").map_or_else(
        || Path::new(".cache").to_path_buf(),
        |proj| proj.cache_dir().to_path_buf(),
    )
}

/// Helper to get the path for a JSON cache file: cache/{category}/{hash}.json
fn get_json_path(category: &str, hash: &str) -> PathBuf {
    cache_root().join(category).join(format!("{hash}.json"))
}

/// Helper to ensure the parent directory exists before writing a file
async fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).await?;
    }

    Ok(())
}

// --- Thumbnails ---

pub async fn get_thumbnail_cache(hash: &str) -> Result<Option<PathBuf>> {
    let thumbs_folder = cache_root().join(THUMBNAILS_DIR).join(hash);
    if !thumbs_folder.exists() {
        return Ok(None);
    }
    Ok(Some(thumbs_folder))
}

pub async fn write_thumbnail_cache(hash: &str, source_folder: &Path) -> Result<()> {
    let dest_folder = cache_root().join(THUMBNAILS_DIR).join(hash);
    if !dest_folder.exists() {
        fs::create_dir_all(&dest_folder).await?;
    }
    copy_dir_contents(source_folder, &dest_folder).await?;
    Ok(())
}

// --- Ingest ---

pub async fn get_ingest_cache(hash: &str) -> Result<Option<MediaMetadata>> {
    let path = get_json_path(INGEST_DIR, hash);
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).await?;
    if let Ok(cached) = serde_json::from_str::<CachedIngestResult>(&data)
        && cached.version == INGEST_CACHE_VERSION
    {
        return Ok(Some(cached.media_metadata));
    }

    warn!("Invalid ingest cache, deleting: {:?}", path);
    let _ = fs::remove_file(&path).await;
    Ok(None)
}

pub async fn write_ingest_cache(hash: &str, media_metadata: MediaMetadata) -> Result<()> {
    let path = get_json_path(INGEST_DIR, hash);
    ensure_parent_dir(&path).await?;

    let json = serde_json::to_string(&CachedIngestResult {
        version: INGEST_CACHE_VERSION,
        media_metadata,
    })?;
    fs::write(path, json).await?;
    Ok(())
}

// --- Analysis ---

pub async fn get_analysis_cache(hash: &str) -> Result<Option<Vec<MLFastAnalysis>>> {
    let path = get_json_path(ANALYSIS_DIR, hash);
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).await?;
    if let Ok(cached) = serde_json::from_str::<CachedAnalysisResult>(&data)
        && cached.version == ANALYSIS_CACHE_VERSION
    {
        if let Some(va) = cached.fast_analyses.first()
            && va.embedding.len() != EXPECTED_EMBEDDING_LENGTH
        {
            return Ok(None);
        }
        return Ok(Some(cached.fast_analyses));
    }

    warn!("Invalid analysis cache, deleting: {:?}", path);
    let _ = fs::remove_file(&path).await;
    Ok(None)
}

pub async fn write_analysis_cache(hash: &str, analyses: &[MLFastAnalysis]) -> Result<()> {
    for analysis in analyses {
        if analysis.embedding.len() != EXPECTED_EMBEDDING_LENGTH {
            bail!(
                "Incorrect embedding length: expected {}, found {}",
                EXPECTED_EMBEDDING_LENGTH,
                analysis.embedding.len()
            );
        }
    }

    let path = get_json_path(ANALYSIS_DIR, hash);
    ensure_parent_dir(&path).await?;

    let json = serde_json::to_string(&CachedAnalysisResult {
        version: ANALYSIS_CACHE_VERSION,
        fast_analyses: Vec::from(analyses),
    })?;
    fs::write(path, json).await?;
    Ok(())
}

// --- LLM ---

pub async fn get_llm_cache(hash: &str) -> Result<Option<Vec<MLChatAnalysis>>> {
    let path = get_json_path(LLM_DIR, hash);
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).await?;
    if let Ok(cached) = serde_json::from_str::<CachedLlmResult>(&data)
        && cached.version == LLM_CACHE_VERSION
    {
        return Ok(Some(cached.llm_analyses));
    }

    warn!("Invalid LLM cache, deleting: {:?}", path);
    let _ = fs::remove_file(&path).await;
    Ok(None)
}

pub async fn write_llm_cache(hash: &str, analyses: &[MLChatAnalysis]) -> Result<()> {
    let path = get_json_path(LLM_DIR, hash);
    ensure_parent_dir(&path).await?;

    let json = serde_json::to_string(&CachedLlmResult {
        version: LLM_CACHE_VERSION,
        llm_analyses: Vec::from(analyses),
    })?;
    fs::write(path, json).await?;
    Ok(())
}
