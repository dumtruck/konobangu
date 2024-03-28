use async_trait::async_trait;
use loco_rs::{
    app::Hooks,
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    prelude::*,
    task::Tasks,
    worker::Processor,
};
use sea_orm::prelude::*;

use crate::{
    controllers,
    migrations::Migrator,
    models::{bangumi, downloaders, episodes, resources, subscribers, subscriptions},
    storage::AppDalInitializer,
    workers::subscription::SubscriptionWorker,
};

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
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

    fn connect_workers<'a>(p: &'a mut Processor, ctx: &'a AppContext) {
        p.register(SubscriptionWorker::build(ctx));
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        futures::try_join!(
            subscribers::Entity::delete_many()
                .filter(subscribers::Column::Id.ne(subscribers::ROOT_SUBSCRIBER_ID))
                .exec(db),
            truncate_table(db, subscriptions::Entity),
            truncate_table(db, resources::Entity),
            truncate_table(db, downloaders::Entity),
            truncate_table(db, bangumi::Entity),
            truncate_table(db, episodes::Entity),
        )?;
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _base: &std::path::Path) -> Result<()> {
        Ok(())
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(AppDalInitializer)])
    }
}
