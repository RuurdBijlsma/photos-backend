use ml_analysis::PyInterop;
use pyo3::prelude::*;
use std::path::Path;
use std::time::Instant;

fn main() -> PyResult<()> {
    Python::attach(|py| {
        let py_interop = PyInterop::new(py)?;
        println!("Python analyzer initialized.");

        let files = vec![Path::new("media_dir/rutenl/ocr-bug.jpg")];

        for file in files {
            // === OCR ===
            let now = Instant::now();
            let ocr_data = py_interop.ocr(file, vec!["nld".to_string(), "eng".to_string()])?;
            println!("\tHas text: {}", ocr_data.has_legible_text);
            if ocr_data.has_legible_text
                && let Some(ocr_text) = ocr_data.ocr_text
            {
                println!("\tOCR text: {ocr_text}");
            }
            println!("OCR took {:#?}.", now.elapsed());
        }

        Ok(())
    })
}
