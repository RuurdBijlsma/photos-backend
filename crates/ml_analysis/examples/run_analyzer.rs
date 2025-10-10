use ml_analysis::VisualAnalyzer;
use pyo3::prelude::*;
use std::path::Path;
use std::time::Instant;

fn main() -> PyResult<()> {
    Python::attach(|py| {
        let analyzer = VisualAnalyzer::new(py)?;
        println!("Python analyzer initialized.");

        let files = vec![
            Path::new("media_dir/sunset.jpg"),
            Path::new("media_dir/tree.jpg"),
            Path::new("media_dir/road.jpg"),
        ];

        for file in files {
            println!("\n\nAnalyzing file: {:?}", file);

            // === CAPTIONER ===
            let now = Instant::now();
            let caption = analyzer.caption_image(file, None)?;
            println!("\tcaption: {}", caption);
            let has_animal = analyzer.caption_image(
                file,
                Some("Question: Is there an animal in the photo? yes or no. Answer:"),
            )?;
            println!("\thas animal: {:#?}", has_animal);
            if has_animal.to_lowercase().contains("yes") {
                let animal_type = analyzer.caption_image(
                    file,
                    Some("Question: What kind of animal is shown in the photo? Answer:"),
                )?;
                println!("\tAnimal type: {}", animal_type);
            }
            let elapsed = now.elapsed();
            println!("Captioner took {:#?}.", elapsed);

            // === FACIAL RECOGNITION ===
            let now = Instant::now();
            let faces = analyzer.facial_recognition(file)?;
            println!("\tFound {:?} faces.", faces.len());
            for face in faces {
                println!("\t=== Found Face ===");
                println!("\tFace sex: {:?}", &face.sex);
                println!("\tFace age: {:?}", &face.age);
            }
            let elapsed = now.elapsed();
            println!("Facial recognition took {:#?}.", elapsed);

            // === OBJECT DETECTION ===
            let now = Instant::now();
            let objects = analyzer.object_detection(file)?;
            println!("\tFound {:?} objects.", objects.len());
            for object in objects {
                println!("\t- Found object, label: {:?}", &object.label);
            }
            let elapsed = now.elapsed();
            println!("Object detection took {:#?}.", elapsed);

            // === OCR ===
            let now = Instant::now();
            let ocr_data = analyzer.ocr(file, vec!["nld".to_string(), "eng".to_string()])?;
            println!("\tHas text: {}", ocr_data.has_legible_text);
            if ocr_data.has_legible_text
                && let Some(ocr_text) = ocr_data.ocr_text
            {
                println!("\tOCR text: {}", ocr_text);
            }
            let elapsed = now.elapsed();
            println!("OCR took {:#?}.", elapsed);
        }

        Ok(())
    })
}
