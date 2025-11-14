use crate::caption_data::get_caption_data;
use crate::color_data::get_color_data;
use crate::quality_data::get_quality_data;
use crate::utils::convert_media_file;
use crate::{ChatMessage, PyInterop, VisualImageData};
use color_eyre::eyre::eyre;
use pyo3::Python;
use serde_json::Value;
use std::path::Path;
use tempfile::Builder;
use common_services::get_settings::settings;
use common_types::Variant;

pub struct VisualAnalyzer {
    py_interop: PyInterop,
}

impl VisualAnalyzer {
    /// Creates a new instance of the `VisualAnalyzer`, initializing the Python interoperability layer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the Python environment cannot be initialized or the required Python modules are not found.
    pub fn new() -> color_eyre::Result<Self> {
        Python::attach(|py| {
            let py_interop = PyInterop::new(py)?;
            Ok(Self { py_interop })
        })
    }

    /// Get theme json from a given color.
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

    /// Send llm message history, and receive a response.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension cannot be determined, if file conversion to JPEG fails, or if any of the underlying analysis steps encounter an error.
    pub fn llm_chat(&self, messages: Vec<ChatMessage>) -> color_eyre::Result<String> {
        let result = self.py_interop.llm_chat(messages)?;
        Ok(result)
    }

    /// Performs a visual analysis of the given image file, extracting various data points like color, quality, and content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension cannot be determined, if file conversion to JPEG fails, or if any of the underlying analysis steps encounter an error.
    pub async fn analyze_image(
        &self,
        file: &Path,
        percentage: i32,
    ) -> color_eyre::Result<VisualImageData> {
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
        let analyzer_settings = &settings().analyzer;

        let color_data = get_color_data(
            &self.py_interop,
            &analysis_file,
            &analyzer_settings.theme_generation.variant,
            analyzer_settings.theme_generation.contrast_level as f32,
        )?;

        let quality_data = get_quality_data(&analysis_file)?;

        let caption_data = get_caption_data(&self.py_interop, &analysis_file)?;

        let embedding = self.py_interop.embed_image(&analysis_file)?;

        let faces = self.py_interop.facial_recognition(&analysis_file)?;

        let objects = self.py_interop.object_detection(&analysis_file)?;

        let ocr = self
            .py_interop
            .ocr(&analysis_file, analyzer_settings.ocr.languages.clone())?;

        // delete the tempfile
        tokio::fs::remove_file(&analysis_file).await?;

        Ok(VisualImageData {
            percentage,
            color_data,
            quality_data,
            caption_data,
            embedding,
            faces,
            objects,
            ocr,
        })
    }
}
