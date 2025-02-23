#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;
mod m20250208_202027_metadata;
mod m20250208_224238_times;
mod m20250208_224457_tags;
mod m20250208_224921_locations;
mod m20250208_225355_gps;
mod m20250208_230751_weather;
mod m20250210_133959_add_unique_faces_table;
mod m20250222_145335_face_boxes;
mod m20250222_165339_ocr_boxes;
mod m20250222_170059_object_boxes;
mod m20250222_215005_add_unique_face_ref_to_face_boxes;
mod m20250222_222637_visual_features;
mod m20250222_232031_fix_locations_unique_constraint;
mod m20250223_103000_add_visual_feature_refs;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20250208_202027_metadata::Migration),
            Box::new(m20250208_224238_times::Migration),
            Box::new(m20250208_224457_tags::Migration),
            Box::new(m20250208_224921_locations::Migration),
            Box::new(m20250208_225355_gps::Migration),
            Box::new(m20250208_230751_weather::Migration),
            Box::new(m20250210_133959_add_unique_faces_table::Migration),
            Box::new(m20250222_145335_face_boxes::Migration),
            Box::new(m20250222_165339_ocr_boxes::Migration),
            Box::new(m20250222_170059_object_boxes::Migration),
            Box::new(m20250222_215005_add_unique_face_ref_to_face_boxes::Migration),
            Box::new(m20250222_222637_visual_features::Migration),
            Box::new(m20250222_232031_fix_locations_unique_constraint::Migration),
            Box::new(m20250223_103000_add_visual_feature_refs::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}