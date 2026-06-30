use color_eyre::Result;
use color_eyre::eyre::bail;
use common_services::caching::cache_root;
use common_types::ml_analysis::{MLChatAnalysis, MLFastAnalysis};
use generate_thumbnails::copy_dir_contents;
use media_analyzer::MediaMetadata;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

// Category folder names
const THUMBNAILS_DIR: &str = "thumbnails";
const INGEST_DIR: &str = "ingest";
const ANALYSIS_DIR: &str = "analysis";
const LLM_DIR: &str = "llm";

const THUMBNAILS_CACHE_VERSION: u32 = 1;
const INGEST_CACHE_VERSION: u32 = 2;
const ANALYSIS_CACHE_VERSION: u32 = 1;
const LLM_CACHE_VERSION: u32 = 1;
const EXPECTED_EMBEDDING_LENGTH: usize = 768;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedThumbnailMetadata {
    pub version: u32,
    pub has_panorama: bool,
}

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

pub async fn get_thumbnail_cache(
    hash: &str,
    thumbnails_dest: &Path,
    pano_dest: &Path,
    require_panorama: bool,
) -> Result<bool> {
    let cache_item_dir = cache_root().join(THUMBNAILS_DIR).join(hash);
    let metadata_path = cache_item_dir.join("metadata.json");

    if !metadata_path.exists() {
        return Ok(false);
    }

    let data = fs::read_to_string(&metadata_path).await?;
    let cached: CachedThumbnailMetadata = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(_) => {
            warn!("Invalid thumbnail cache metadata, deleting: {:?}", cache_item_dir);
            let _ = fs::remove_dir_all(&cache_item_dir).await;
            return Ok(false);
        }
    };

    // Check version
    if cached.version != THUMBNAILS_CACHE_VERSION {
        warn!(
            "Thumbnail cache version mismatch (expected {}, found {}), deleting: {:?}",
            THUMBNAILS_CACHE_VERSION, cached.version, cache_item_dir
        );
        let _ = fs::remove_dir_all(&cache_item_dir).await;
        return Ok(false);
    }

    // Ensure we aren't using a non-pano cache if a panorama is required
    if require_panorama && !cached.has_panorama {
        debug!("Cache exists but does not contain required panorama.");
        return Ok(false);
    }

    // Restore thumbnails folder
    let cached_thumbs = cache_item_dir.join("thumbs");
    if cached_thumbs.exists() {
        if !thumbnails_dest.exists() {
            fs::create_dir_all(thumbnails_dest).await?;
        }
        copy_dir_contents(&cached_thumbs, thumbnails_dest).await?;
    } else {
        return Ok(false);
    }

    // Restore panorama folder if it was cached
    if cached.has_panorama {
        let cached_pano = cache_item_dir.join("pano");
        if cached_pano.exists() {
            if !pano_dest.exists() {
                fs::create_dir_all(pano_dest).await?;
            }
            copy_dir_contents(&cached_pano, pano_dest).await?;
        } else {
            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn write_thumbnail_cache(
    hash: &str,
    thumbnails_src: &Path,
    pano_src: &Path,
    has_panorama: bool,
) -> Result<()> {
    let cache_item_dir = cache_root().join(THUMBNAILS_DIR).join(hash);
    if cache_item_dir.exists() {
        fs::remove_dir_all(&cache_item_dir).await?;
    }
    fs::create_dir_all(&cache_item_dir).await?;

    // Copy thumbnails to cache
    let cached_thumbs = cache_item_dir.join("thumbs");
    fs::create_dir_all(&cached_thumbs).await?;
    copy_dir_contents(thumbnails_src, &cached_thumbs).await?;

    // Copy panorama to cache if requested and output directory exists
    let actual_has_panorama = has_panorama && pano_src.exists();
    if actual_has_panorama {
        let cached_pano = cache_item_dir.join("pano");
        fs::create_dir_all(&cached_pano).await?;
        copy_dir_contents(pano_src, &cached_pano).await?;
    }

    // Write metadata file
    let metadata_path = cache_item_dir.join("metadata.json");
    let metadata = CachedThumbnailMetadata {
        version: THUMBNAILS_CACHE_VERSION,
        has_panorama: actual_has_panorama,
    };
    let json = serde_json::to_string(&metadata)?;
    fs::write(metadata_path, json).await?;

    Ok(())
}

pub async fn delete_thumbnail_cache(hash: &str) -> Result<()> {
    let dest_folder = cache_root().join(THUMBNAILS_DIR).join(hash);
    if dest_folder.exists() {
        fs::remove_dir_all(&dest_folder).await?;
    }
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
