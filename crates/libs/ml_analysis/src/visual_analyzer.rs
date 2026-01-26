use crate::PyInterop;
use crate::caption_data::get_caption_data;
use crate::color_data::get_color_data;
use crate::quality_judge::get_quality_judgement;
use crate::quality_measure::get_quality_measurement;
use crate::utils::convert_media_file;
use app_state::AnalyzerSettings;
use color_eyre::eyre::eyre;
use common_types::ml_analysis::{CombinedQuality, RawVisualAnalysis};
use common_types::variant::Variant;
use language_model::{ChatSession, LlamaClient};
use pyo3::Python;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tempfile::Builder;

pub struct VisualAnalyzer {
    py_interop: PyInterop,
    pub llm_client: LlamaClient,
}

impl VisualAnalyzer {
    /// Creates a new instance of the `VisualAnalyzer`, initializing the Python interoperability layer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the Python environment cannot be initialized or the required Python modules are not found.
    pub fn new() -> color_eyre::Result<Self> {
        let llm = LlamaClient::with_base_url("http://localhost:8080").build();
        Python::attach(|py| {
            let py_interop = PyInterop::new(py)?;
            Ok(Self {
                py_interop,
                llm_client: llm,
            })
        })
    }

    /// Get stateful llm session from llm client
    #[must_use]
    pub fn get_llm_session(&self) -> ChatSession {
        ChatSession::new(self.llm_client.clone())
    }

    /// Get theme JSON from a given color.
    ///
    /// # Errors
    ///
    /// Error if we can't get theme from color, or Python interop don't work.
    pub fn theme_from_color(
        &self,
        color: &str,
        variant: &Variant,
        contrast_level: f32,
    ) -> color_eyre::Result<Value> {
        let result = self
            .py_interop
            .get_theme_from_color(color, variant, contrast_level)?;
        Ok(result)
    }

    /// Performs a visual analysis of the given image file, extracting various data points like color, quality, and content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension cannot be determined, if file conversion to JPEG fails, or if any of the underlying analysis steps encounter an error.
    pub async fn analyze_image(
        &self,
        config: &AnalyzerSettings,
        file: &Path,
        percentage: i32,
    ) -> color_eyre::Result<RawVisualAnalysis> {
        let start = Instant::now();
        let now = Instant::now();
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
            convert_media_file(file, &analysis_file).await?;
        }
        println!("Convert to jpg {:?}", now.elapsed());

        let now = Instant::now();
        let color_data = get_color_data(
            &self.py_interop,
            &analysis_file,
            &config.theme_generation.variant,
            config.theme_generation.contrast_level as f32,
        )?;
        println!("get_color_data {:?}", now.elapsed());

        let now = Instant::now();
        let categorization_data = get_caption_data(&self.llm_client, &analysis_file).await?;
        println!("get_caption_data {:?}", now.elapsed());

        let now = Instant::now();
        let quality_measurement = get_quality_measurement(&analysis_file)?;
        println!("get_quality_measurement {:?}", now.elapsed());

        let now = Instant::now();
        let quality_judgement = get_quality_judgement(&self.llm_client, &analysis_file).await?;
        println!("get_quality_judgement {:?}", now.elapsed());

        let quality = CombinedQuality {
            measured: quality_measurement,
            judged: quality_judgement,
        };

        let now = Instant::now();
        let embedding = self.py_interop.embed_image(&analysis_file)?;
        println!("embed_image {:?}", now.elapsed());

        let now = Instant::now();
        let _x = self
            .py_interop
            .embed_text("This is my search query i want an image")?;
        println!("embed_text {:?}", now.elapsed());

        let now = Instant::now();
        let faces = self.py_interop.facial_recognition(&analysis_file)?;
        println!("facial_recognition {:?}", now.elapsed());

        let now = Instant::now();
        let objects = self.py_interop.object_detection(&analysis_file)?;
        println!("object_detection {:?}", now.elapsed());

        tokio::fs::remove_file(&analysis_file).await?;

        println!("total ml analysis {:?}", start.elapsed());

        Ok(RawVisualAnalysis {
            percentage,
            color_data,
            quality,
            categorization_data,
            embedding,
            faces,
            objects,
        })
    }
}
