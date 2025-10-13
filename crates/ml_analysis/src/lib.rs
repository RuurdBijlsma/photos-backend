#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

mod caption_data;
mod color_data;
mod py_interop;
mod quality_data;
mod structs;
mod utils;
mod visual_analyzer;

pub use color_data::get_color_data;
pub use py_interop::PyInterop;
pub use quality_data::get_quality_data;
pub use structs::*;
pub use visual_analyzer::VisualAnalyzer;