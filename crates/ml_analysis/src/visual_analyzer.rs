use crate::structs::{FaceBox, OCRData, ObjectBox};
use pyo3::prelude::*;
use std::path::Path;

pub struct VisualAnalyzer {
    json_dumps: Py<PyAny>,
    facial_recognition_func: Py<PyAny>,
    captioner_func: Py<PyAny>,
    object_detection_func: Py<PyAny>,
    ocr_func: Py<PyAny>,
}

impl VisualAnalyzer {
    /// Initializes the `VisualAnalyzer` by loading and preparing all the necessary Python machine learning models and functions.
    ///
    /// # Errors
    ///
    /// This function will return an error if the Python interpreter cannot be initialized, required Python modules (`sys`, `os`, `json`, `py_analyze`) cannot be imported, or necessary functions cannot be found within the `py_analyze` module.
    pub fn new(py: Python<'_>) -> PyResult<Self> {
        let sys = py.import("sys")?;
        let sys_path = sys.getattr("path")?;
        sys_path.call_method1("append", ("./crates/ml_analysis/py_ml",))?;
        sys_path.call_method1(
            "append",
            ("./crates/ml_analysis/py_ml/.venv/Lib/site-packages",),
        )?;

        // --- Python log suppression code ---
        let builtins = py.import("builtins")?;
        let open = builtins.getattr("open")?;
        let os = py.import("os")?;
        let devnull_path = os.getattr("devnull")?;
        let devnull_file = open.call1((devnull_path, "w"))?;
        sys.setattr("stdout", devnull_file.clone())?;
        sys.setattr("stderr", devnull_file)?;
        // --- End of log suppression code ---

        let module = py.import("py_analyze")?;
        let captioner_func = module.getattr("caption")?.into_pyobject(py)?;
        let facial_recognition_func = module.getattr("facial_recognition")?.into_pyobject(py)?;
        let object_detection_func = module.getattr("object_detection")?.into_pyobject(py)?;
        let ocr_func = module.getattr("ocr")?.into_pyobject(py)?;

        let json = py.import("json")?;
        let dumps = json.getattr("dumps")?.into_pyobject(py)?;

        Ok(Self {
            ocr_func: ocr_func.into(),
            object_detection_func: object_detection_func.into(),
            facial_recognition_func: facial_recognition_func.into(),
            captioner_func: captioner_func.into(),
            json_dumps: dumps.into(),
        })
    }

    /// Generates a descriptive caption for the given image, with an option to provide a specific instructional prompt.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying Python captioning function fails, which could be due to an invalid image path, a model inference error, or if the result cannot be converted to a Rust `String`.
    pub fn caption_image(&self, image: &Path, instruction: Option<&str>) -> Result<String, PyErr> {
        Python::attach(|py| {
            let func = self.captioner_func.bind(py);

            let result = func.call1((image, instruction))?;
            result.extract()
        })
    }

    /// Performs facial recognition on the provided image to detect and analyze any faces present.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying Python facial recognition function fails or if the returned data cannot be serialized to a JSON string.
    ///
    /// # Panics
    ///
    /// This function will panic if the JSON string returned from the Python function does not correctly deserialize into the `Vec<FaceBox>` struct, indicating a mismatch between the Python and Rust data structures.
    pub fn facial_recognition(&self, image: &Path) -> Result<Vec<FaceBox>, PyErr> {
        Python::attach(|py| {
            let func = self.facial_recognition_func.bind(py);
            let dumps = self.json_dumps.bind(py);

            let result = func.call1((image,))?;
            let json_str: String = dumps.call1((result,))?.extract()?;
            let faces: Vec<FaceBox> = serde_json::from_str(&json_str).unwrap();
            Ok(faces)
        })
    }

    /// Detects objects within the given image and returns a list of their labels and bounding boxes.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying Python object detection function fails or if the returned data cannot be serialized to a JSON string.
    ///
    /// # Panics
    ///
    /// This function will panic if the JSON string returned from the Python function does not correctly deserialize into the `Vec<ObjectBox>` struct, indicating a mismatch between the Python and Rust data structures.
    pub fn object_detection(&self, image: &Path) -> Result<Vec<ObjectBox>, PyErr> {
        Python::attach(|py| {
            let func = self.object_detection_func.bind(py);
            let dumps = self.json_dumps.bind(py);

            let result = func.call1((image,))?;
            let json_str: String = dumps.call1((result,))?.extract()?;
            let objects: Vec<ObjectBox> = serde_json::from_str(&json_str).unwrap();
            Ok(objects)
        })
    }

    /// Performs Optical Character Recognition (OCR) on an image to extract text, given a list of target languages.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying Python OCR function fails or if the returned data cannot be serialized to a JSON string.
    ///
    /// # Panics
    ///
    /// This function will panic if the JSON string returned from the Python function does not correctly deserialize into the `OCRData` struct, indicating a mismatch between the Python and Rust data structures.
    pub fn ocr(&self, image: &Path, languages: Vec<String>) -> Result<OCRData, PyErr> {
        Python::attach(|py| {
            let func = self.ocr_func.bind(py);
            let dumps = self.json_dumps.bind(py);

            let result = func.call1((image, languages))?;
            let json_str: String = dumps.call1((result,))?.extract()?;
            let ocr_data: OCRData = serde_json::from_str(&json_str).unwrap();
            Ok(ocr_data)
        })
    }
}