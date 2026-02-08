use image::GenericImageView;
use image::imageops::FilterType;
use language_model::LlamaClient;
use ml_analysis::get_caption_data;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tempfile::Builder;
use walkdir::WalkDir;

fn resize_image(input: &Path, max_dim: u32) -> image::ImageResult<PathBuf> {
    let img = image::open(input)?;
    let (w, h) = img.dimensions();
    let scale = (max_dim as f32 / w.max(h) as f32).min(1.0);
    let new_w = (w as f32 * scale).round() as u32;
    let new_h = (h as f32 * scale).round() as u32;
    let resized = img.resize(new_w, new_h, FilterType::CatmullRom);
    let tmp = Builder::new()
        .suffix(".jpg")
        .tempfile()
        .map_err(image::ImageError::IoError)?;
    resized.save(tmp.path())?;
    let (_file, path) = tmp
        .keep()
        .map_err(|e| image::ImageError::IoError(e.error))?;
    Ok(path)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let llm_client = LlamaClient::with_base_url("http://localhost:8080").build();
    let root_folder = "C:/Users/Ruurd/Pictures/small_media_dir/rutenl";

    for entry in WalkDir::new(root_folder).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let image = entry.path();
            let image_filename = image.file_name().unwrap().to_string_lossy().to_string();
            let resized_img_file = resize_image(image, 720)?;

            let now = Instant::now();
            let caption_data = get_caption_data(&llm_client, &resized_img_file).await?;

            if let Some(cd) = caption_data {
                let filename = format!("{image_filename}-caption.json");
                let file = File::create(Path::new(&filename))?;
                serde_json::to_writer_pretty(file, &cd)?;
                println!("{image_filename} {:?}", now.elapsed());
            } else {
                eprintln!("Couldn't get caption data");
            }
        }
    }

    Ok(())
}
