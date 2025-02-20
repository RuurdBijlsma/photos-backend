#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;
mod m20250208_202027_metadata;
mod m20250208_224238_times;
mod m20250208_224457_tags;
mod m20250208_224921_locations;
mod m20250208_225355_gps;
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
            // inject-above (do not remove this comment)
        ]
    }
}