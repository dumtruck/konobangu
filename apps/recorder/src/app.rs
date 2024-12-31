use std::path::Path;

use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    cache,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    prelude::*,
    task::Tasks,
    Result,
};
use sea_orm::DatabaseConnection;

use crate::{
    auth::service::AppAuthService,
    controllers,
    dal::{AppDalClient, AppDalInitalizer},
    extract::mikan::{client::AppMikanClientInitializer, AppMikanClient},
    migrations::Migrator,
    models::entities::subscribers,
    workers::subscription_worker::SubscriptionWorker,
};

pub trait AppContextExt {
    fn get_dal_client(&self) -> &AppDalClient {
        AppDalClient::app_instance()
    }

    fn get_mikan_client(&self) -> &AppMikanClient {
        AppMikanClient::app_instance()
    }

    fn get_auth_service(&self) -> &AppAuthService {
        AppAuthService::app_instance()
    }
}

impl AppContextExt for AppContext {}

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        let initializers: Vec<Box<dyn Initializer>> = vec![
            Box::new(AppDalInitalizer),
            Box::new(AppMikanClientInitializer),
        ];

        Ok(initializers)
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .prefix("/api")
            .add_route(controllers::subscribers::routes())
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(SubscriptionWorker::build(ctx)).await?;
        Ok(())
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        Ok(AppContext {
            cache: cache::Cache::new(cache::drivers::inmem::new()).into(),
            ..ctx
        })
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        truncate_table(db, subscribers::Entity).await?;
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _base: &Path) -> Result<()> {
        Ok(())
    }
}
