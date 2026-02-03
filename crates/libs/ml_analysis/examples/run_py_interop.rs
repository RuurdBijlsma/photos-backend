use ml_analysis::PyInterop;
use pyo3::prelude::*;
use std::path::Path;
use std::time::Instant;

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
        }

        Ok(())
    })
}
