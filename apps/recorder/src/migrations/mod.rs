pub use sea_orm_migration::prelude::*;

#[macro_use]
pub mod defs;
pub mod m20220101_000001_init;
pub mod m20240224_082543_add_downloads;
pub mod m20241231_000001_auth;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_init::Migration),
            Box::new(m20240224_082543_add_downloads::Migration),
            Box::new(m20241231_000001_auth::Migration),
        ]
    }
}
