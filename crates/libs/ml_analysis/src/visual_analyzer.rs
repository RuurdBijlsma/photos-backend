use crate::color_data::get_color_data;
use crate::get_llm_classification;
use crate::quality_judge::get_quality_judgement;
use crate::quality_measure::get_quality_measurement;
use crate::utils::convert_media_file;
use app_state::AnalyzerSettings;
use color_eyre::eyre::eyre;
use common_types::ml_analysis::{MLChatAnalysis, MLCombinedQuality, MLFastAnalysis};
use face_id::analyzer::FaceAnalyzer;
use language_model::{ChatSession, LlamaClient};
use open_clip_inference::VisionEmbedder;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::Builder;

pub struct VisualAnalyzer {
    pub embedder: VisionEmbedder,
    pub llm_client: LlamaClient,
    pub face_analyzer: FaceAnalyzer,
}

impl VisualAnalyzer {
    /// Creates a new instance of the `VisualAnalyzer`.
    pub async fn new(embedder_model_id: &str) -> color_eyre::Result<Self> {
        let embedder = VisionEmbedder::from_hf(embedder_model_id).build().await?;
        let llm = LlamaClient::with_base_url("http://localhost:8080").build();
        let face_analyzer = FaceAnalyzer::from_hf().build().await?;
        Ok(Self {
            llm_client: llm,
            embedder,
            face_analyzer,
        })
    }

    /// Get stateful llm session from llm client
    #[must_use]
    pub fn get_llm_session(&self) -> ChatSession {
        ChatSession::new(self.llm_client.clone())
    }

    async fn get_analysis_file(
        file: &Path,
        analyze_image_size: u64,
    ) -> color_eyre::Result<PathBuf> {
        let Some(extension) = file.extension().map(|e| e.to_string_lossy().to_string()) else {
            return Err(eyre!("Can't get extension from file"));
        };

        let mut analysis_file = file.to_path_buf();
        if !["jpg", "jpeg"].contains(&&*extension.to_lowercase()) {
            let temp_file = Builder::new()
                .suffix(".jpg")
                .disable_cleanup(true)
                .tempfile()?;
            analysis_file = temp_file.path().to_path_buf();
            convert_media_file(file, &analysis_file, analyze_image_size).await?;
        }
        Ok(analysis_file)
    }

    /// Performs a visual analysis of the given image file, extracting various data points like color, quality, and content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension cannot be determined, if file conversion to JPEG fails, or if any of the underlying analysis steps encounter an error.
    pub async fn fast_image_analysis(
        &self,
        config: &AnalyzerSettings,
        file: &Path,
        percentage: i32,
    ) -> color_eyre::Result<MLFastAnalysis> {
        let start = Instant::now();
        let now = Instant::now();
        let analysis_file = Self::get_analysis_file(file, config.analyze_image_size).await?;
        println!("Convert to jpg {:?}", now.elapsed());

        let now = Instant::now();
        let color_data = get_color_data(
            &analysis_file,
            &config.theme_generation.variant,
            config.theme_generation.contrast_level,
        )?;
        println!("get_color_data {:?}", now.elapsed());

        let img = image::open(&analysis_file)?;

        let now = Instant::now();
        let embedding = self.embedder.embed_image(&img)?.to_vec();
        println!("embed_image {:?}", now.elapsed());

        let now = Instant::now();
        let faces = self.face_analyzer.analyze(&img)?;
        println!("facial_recognition {:?}", now.elapsed());

        tokio::fs::remove_file(&analysis_file).await?;

        println!("total ml analysis {:?}", start.elapsed());

        Ok(MLFastAnalysis {
            percentage,
            color_data,
            embedding,
            faces,
        })
    }

    /// Performs a visual analysis of the given image file, extracting various data points like color, quality, and content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension cannot be determined, if file conversion to JPEG fails, or if any of the underlying analysis steps encounter an error.
    pub async fn llm_analysis(
        &self,
        config: &AnalyzerSettings,
        file: &Path,
        percentage: i32,
    ) -> color_eyre::Result<MLChatAnalysis> {
        let start = Instant::now();
        let now = Instant::now();
        let analysis_file = Self::get_analysis_file(file, config.analyze_image_size).await?;
        println!("Convert to jpg {:?}", now.elapsed());

        let now = Instant::now();
        let llm_classification = get_llm_classification(&self.llm_client, &analysis_file).await?;
        println!("get_caption_data {:?}", now.elapsed());

        let now = Instant::now();
        let quality_measurement = get_quality_measurement(&analysis_file)?;
        println!("get_quality_measurement {:?}", now.elapsed());

        let now = Instant::now();
        let quality_judgement = get_quality_judgement(&self.llm_client, &analysis_file).await?;
        println!("get_quality_judgement {:?}", now.elapsed());

        let quality = MLCombinedQuality {
            measured: quality_measurement,
            judged: quality_judgement,
        };

        tokio::fs::remove_file(&analysis_file).await?;

        println!("total ml analysis {:?}", start.elapsed());

        Ok(MLChatAnalysis {
            percentage,
            quality,
            llm_classification,
        })
    }
}
