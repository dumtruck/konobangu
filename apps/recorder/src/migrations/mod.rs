pub use sea_orm_migration::prelude::*;

#[macro_use]
pub mod defs;
pub mod m20220101_000001_init;
pub mod m20240224_082543_add_downloads;
pub mod m20240225_060853_subscriber_add_downloader;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_init::Migration),
            Box::new(m20240224_082543_add_downloads::Migration),
            Box::new(m20240225_060853_subscriber_add_downloader::Migration),
        ]
    }
}
