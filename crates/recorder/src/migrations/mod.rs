pub use sea_orm_migration::prelude::*;

pub mod defs;
pub mod m20220101_000001_init;
pub mod m20240224_082543_add_downloads;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_init::Migration),
            Box::new(m20240224_082543_add_downloads::Migration),
        ]
    }
}
