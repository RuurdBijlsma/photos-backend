use crate::caption_data::get_caption_data;
use crate::color_data::analyze_colors;
use crate::{PyInterop, Variant, VisualImageData};
use pyo3::Python;
use std::path::Path;
use std::time::Instant;

pub struct VisualAnalyzer {
    py_interop: PyInterop,
}

impl VisualAnalyzer {
    pub fn new() -> color_eyre::Result<VisualAnalyzer> {
        Python::attach(|py| {
            let py_interop = PyInterop::new(py)?;
            Ok(Self { py_interop })
        })
    }

    pub fn analyze_image(&self, file: &Path) -> color_eyre::Result<VisualImageData> {
        let now = Instant::now();
        // todo config variant & contrast level
        let color_data = analyze_colors(&self.py_interop, file, &Variant::Vibrant, 3.)?;
        println!("\tanalyze_colors {:?}", now.elapsed());

        let now = Instant::now();
        let caption_data = get_caption_data(&self.py_interop, file)?;
        println!("\tget_caption_data {:?}", now.elapsed());

        let now = Instant::now();
        let embedding = self.py_interop.embed_image(file)?;
        println!("\tembed_image {:?}", now.elapsed());

        let now = Instant::now();
        let faces = self.py_interop.facial_recognition(file)?;
        println!("\tfacial_recognition {:?}", now.elapsed());

        let now = Instant::now();
        let objects = self.py_interop.object_detection(file)?;
        println!("\tobject_detection {:?}", now.elapsed());

        let now = Instant::now();
        // todo config languages
        let ocr = self
            .py_interop
            .ocr(file, vec!["nld".to_string(), "eng".to_string()])?;
        println!("\tocr {:?}", now.elapsed());

        Ok(VisualImageData {
            color_data,
            caption_data,
            embedding,
            faces,
            objects,
            ocr,
        })
    }
}
