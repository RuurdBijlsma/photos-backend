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

const THUMBNAILS_FOLDER: &str = "thumbnails";
const INGEST_RESULT_FILENAME: &str = "ingest_result.json";
const ANALYSIS_RESULT_FILENAME: &str = "analysis_result.json";
const LLM_RESULT_FILENAME: &str = "llm_result.json";
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

pub async fn get_ingest_cache(hash: &str) -> Result<Option<MediaMetadata>> {
    let process_cache_file = cache_folder().join(hash).join(INGEST_RESULT_FILENAME);
    if !process_cache_file.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&process_cache_file).await?;
    if let Ok(cached_ingest) = serde_json::from_str::<CachedIngestResult>(&data)
        && cached_ingest.version == INGEST_CACHE_VERSION
    {
        return Ok(Some(cached_ingest.media_metadata));
    }
    warn!(
        "Found invalid cache file for ingest. Deleting {}/{}.",
        hash, INGEST_RESULT_FILENAME
    );
    fs::remove_file(&process_cache_file).await?;
    Ok(None)
}

pub async fn write_ingest_cache(hash: &str, media_metadata: MediaMetadata) -> Result<()> {
    let hash_folder = hash_cache_folder(hash).await?;
    let ingest_cache_file = hash_folder.join(INGEST_RESULT_FILENAME);
    let json = serde_json::to_string(&CachedIngestResult {
        version: INGEST_CACHE_VERSION,
        media_metadata,
    })?;
    fs::write(ingest_cache_file, json).await?;
    Ok(())
}

pub async fn get_analysis_cache(hash: &str) -> Result<Option<Vec<MLFastAnalysis>>> {
    let process_cache_file = cache_folder().join(hash).join(ANALYSIS_RESULT_FILENAME);
    if !process_cache_file.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&process_cache_file).await?;
    if let Ok(cached_analysis) = serde_json::from_str::<CachedAnalysisResult>(&data)
        && cached_analysis.version == ANALYSIS_CACHE_VERSION
    {
        if let Some(va) = cached_analysis.fast_analyses.first()
            && va.embedding.len() != EXPECTED_EMBEDDING_LENGTH
        {
            return Ok(None);
        }
        return Ok(Some(cached_analysis.fast_analyses));
    }
    warn!(
        "Found invalid cache file for analysis. Deleting {}/{}.",
        hash, ANALYSIS_RESULT_FILENAME
    );
    fs::remove_file(&process_cache_file).await?;
    Ok(None)
}

pub async fn write_analysis_cache(hash: &str, analyses: &[MLFastAnalysis]) -> Result<()> {
    for analysis in analyses {
        if analysis.embedding.len() != EXPECTED_EMBEDDING_LENGTH {
            bail!(
                "Trying to write incorrect embedding length to cache. Expected: {}, found {}",
                EXPECTED_EMBEDDING_LENGTH,
                analysis.embedding.len()
            );
        }
    }
    let hash_folder = hash_cache_folder(hash).await?;
    let ingest_cache_file = hash_folder.join(ANALYSIS_RESULT_FILENAME);
    let json = serde_json::to_string(&CachedAnalysisResult {
        version: ANALYSIS_CACHE_VERSION,
        fast_analyses: Vec::from(analyses),
    })?;
    fs::write(ingest_cache_file, json).await?;
    Ok(())
}

pub async fn get_llm_cache(hash: &str) -> Result<Option<Vec<MLChatAnalysis>>> {
    let process_cache_file = cache_folder().join(hash).join(LLM_RESULT_FILENAME);
    if !process_cache_file.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&process_cache_file).await?;
    if let Ok(cached_analysis) = serde_json::from_str::<CachedLlmResult>(&data)
        && cached_analysis.version == LLM_CACHE_VERSION
    {
        return Ok(Some(cached_analysis.llm_analyses));
    }
    warn!(
        "Found invalid cache file for llm analysis. Deleting {}/{}.",
        hash, LLM_RESULT_FILENAME
    );
    fs::remove_file(&process_cache_file).await?;
    Ok(None)
}

pub async fn write_llm_cache(hash: &str, analyses: &[MLChatAnalysis]) -> Result<()> {
    let hash_folder = hash_cache_folder(hash).await?;
    let ingest_cache_file = hash_folder.join(LLM_RESULT_FILENAME);
    let json = serde_json::to_string(&CachedLlmResult {
        version: LLM_CACHE_VERSION,
        llm_analyses: Vec::from(analyses),
    })?;
    fs::write(ingest_cache_file, json).await?;
    Ok(())
}
