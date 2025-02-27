pub mod builder;
pub mod config;
pub mod context;
pub mod core;
pub mod env;
pub mod router;

pub use core::App;
use std::path::Path;

use async_trait::async_trait;
pub use context::AppContext;
use loco_rs::{
    Result,
    app::{AppContext as LocoAppContext, Hooks},
    boot::{BootResult, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    prelude::*,
    task::Tasks,
};

use crate::{migrations::Migrator, models::subscribers};

pub struct App1;

#[async_trait]
impl Hooks for App1 {
    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &LocoAppContext) -> Result<Vec<Box<dyn Initializer>>> {
        let initializers: Vec<Box<dyn Initializer>> = vec![];

        Ok(initializers)
    }

    fn routes(_ctx: &LocoAppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(ctx: &LocoAppContext) -> Result<()> {
        truncate_table(&ctx.db, subscribers::Entity).await?;
        Ok(())
    }

    async fn seed(_ctx: &LocoAppContext, _base: &Path) -> Result<()> {
        Ok(())
    }

    async fn connect_workers(_ctx: &LocoAppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }
}
