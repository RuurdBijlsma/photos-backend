use crate::caption_data::get_caption_data;
use crate::color_data::get_color_data;
use crate::quality_data::get_quality_data;
use crate::utils::convert_media_file;
use crate::{PyInterop, VisualImageData};
use color_eyre::eyre::eyre;
use common_photos::settings;
use pyo3::Python;
use std::path::Path;
use std::time::Instant;
use tempfile::Builder;

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

    /// Performs a visual analysis of the given image file, extracting various data points like color, quality, and content.
    ///
    /// # Errors
    ///
    /// Returns an error if the file extension cannot be determined, if file conversion to JPEG fails, or if any of the underlying analysis steps encounter an error.
    pub async fn analyze_image(&self, file: &Path) -> color_eyre::Result<VisualImageData> {
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

        let now = Instant::now();
        // todo config variant & contrast level
        let color_data = get_color_data(
            &self.py_interop,
            &analysis_file,
            &analyzer_settings.theme_generation.variant,
            analyzer_settings.theme_generation.contrast_level as f32,
        )?;
        println!("\tget_color_data {:?}", now.elapsed());

        let now = Instant::now();
        let quality_data = get_quality_data(&analysis_file)?;
        println!("\tget_quality_data {:?}", now.elapsed());

        let now = Instant::now();
        let caption_data = get_caption_data(&self.py_interop, &analysis_file)?;
        println!("\tget_caption_data {:?}", now.elapsed());

        let now = Instant::now();
        let embedding = self.py_interop.embed_image(&analysis_file)?;
        println!("\tembed_image {:?}", now.elapsed());

        let now = Instant::now();
        let faces = self.py_interop.facial_recognition(&analysis_file)?;
        println!("\tfacial_recognition {:?}", now.elapsed());

        let now = Instant::now();
        let objects = self.py_interop.object_detection(&analysis_file)?;
        println!("\tobject_detection {:?}", now.elapsed());

        let now = Instant::now();
        let ocr = self
            .py_interop
            .ocr(file, analyzer_settings.ocr.languages.clone())?;
        println!("\tocr {:?}", now.elapsed());

        // delete the tempfile
        tokio::fs::remove_file(&analysis_file).await?;

        Ok(VisualImageData {
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
