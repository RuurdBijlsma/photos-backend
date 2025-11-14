use ml_analysis::PyInterop;
use pyo3::prelude::*;
use std::path::Path;
use std::time::Instant;

/// Calculates the cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        0.0
    } else {
        dot_product / (magnitude_a * magnitude_b)
    }
}

fn main() -> PyResult<()> {
    Python::attach(|py| {
        let py_interop = PyInterop::new(py)?;
        println!("Python analyzer initialized.");

        let files = vec![
            Path::new("media_dir/sunset.jpg"),
            Path::new("media_dir/tree.jpg"),
            Path::new("media_dir/road.jpg"),
        ];

        for file in files {
            println!("\n\nAnalyzing file: {}", file.display());

            // === EMBEDDER ===
            let now = Instant::now();
            let image_embedding = py_interop.embed_image(file)?;

            // Define the texts and embed them all in a single batch
            let texts_to_compare = vec![
                "This is an image of a road during a sunset.",
                "A photo of a tree.",
                "This is a sunset.",
                "Photo of a sunset taken from a boat. A crane is in the foreground.",
            ];
            let text_embeddings = py_interop.embed_texts(texts_to_compare.clone())?;

            // Calculate similarity score for each text against the image
            let mut similarities: Vec<(&str, f32)> = Vec::new();
            for (i, text_embedding) in text_embeddings.iter().enumerate() {
                let similarity = cosine_similarity(&image_embedding, text_embedding);
                similarities.push((texts_to_compare[i], similarity));
            }

            // Sort by similarity score in descending order
            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            // Print the sorted results
            println!("\tText similarity scores (higher is better):");
            for (text, score) in similarities {
                println!("\t- Score: {score:.4}, Text: \"{text}\"");
            }
            println!("Embedder took {:#?}.", now.elapsed());

            // === CAPTIONER ===
            let now = Instant::now();
            let caption = py_interop.caption_image(file, None)?;
            println!("\tcaption: {caption}");
            let has_animal = py_interop.caption_image(
                file,
                Some("Question: Is there an animal in the photo? yes or no. Answer:"),
            )?;
            println!("\thas animal: {has_animal:#?}");
            if has_animal.to_lowercase().contains("yes") {
                let animal_type = py_interop.caption_image(
                    file,
                    Some("Question: What kind of animal is shown in the photo? Answer:"),
                )?;
                println!("\tAnimal type: {animal_type}");
            }
            println!("Captioner took {:#?}.", now.elapsed());

            // === FACIAL RECOGNITION ===
            let now = Instant::now();
            let faces = py_interop.facial_recognition(file)?;
            println!("\tFound {:?} faces.", faces.len());
            for face in faces {
                println!("\t=== Found Face ===");
                println!("\tFace sex: {:?}", &face.sex);
                println!("\tFace age: {:?}", &face.age);
            }
            println!("Facial recognition took {:#?}.", now.elapsed());

            // === OBJECT DETECTION ===
            let now = Instant::now();
            let objects = py_interop.object_detection(file)?;
            println!("\tFound {:?} objects.", objects.len());
            for object in objects {
                println!("\t- Found object, label: {:?}", &object.label);
            }
            println!("Object detection took {:#?}.", now.elapsed());

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
