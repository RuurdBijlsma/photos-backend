#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_sign_loss,
    clippy::module_inception,
    clippy::struct_excessive_bools,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

pub mod api;
pub mod database;
pub mod get_settings;
pub mod job_queue;
pub mod utils;
pub mod s2s_client;