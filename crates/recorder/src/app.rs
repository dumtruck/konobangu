use async_trait::async_trait;
use loco_rs::{
    app::Hooks,
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    prelude::*,
    task::Tasks,
    worker::Processor,
};
use sea_orm::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::{
    controllers,
    migrations::Migrator,
    models::{bangumi, downloaders, episodes, resources, subscribers, subscriptions},
    storage::AppDalInitializer,
    utils::cli::hack_env_to_fit_workspace,
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
        hack_env_to_fit_workspace()?;
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
                .filter(subscribers::Column::Pid.ne(subscribers::ROOT_SUBSCRIBER_NAME))
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

    fn init_logger(app_config: &Config, _env: &Environment) -> Result<bool> {
        let config = &app_config.logger;
        if config.enable {
            let filter = EnvFilter::try_from_default_env()
                .or_else(|_| {
                    // user wanted a specific filter, don't care about our internal whitelist
                    // or, if no override give them the default whitelisted filter (most common)
                    config.override_filter.as_ref().map_or_else(
                        || {
                            EnvFilter::try_new(
                                ["loco_rs", "sea_orm_migration", "tower_http", "sqlx::query"]
                                    .iter()
                                    .map(|m| format!("{}={}", m, config.level))
                                    .chain(std::iter::once(format!(
                                        "{}={}",
                                        App::app_name(),
                                        config.level
                                    )))
                                    .collect::<Vec<_>>()
                                    .join(","),
                            )
                        },
                        EnvFilter::try_new,
                    )
                })
                .expect("logger initialization failed");

            let builder = tracing_subscriber::FmtSubscriber::builder().with_env_filter(filter);

            match serde_json::to_string(&config.format)
                .expect("init logger format can serialized")
                .trim_matches('"')
            {
                "pretty" => builder.pretty().init(),
                "json" => builder.json().init(),
                _ => builder.compact().init(),
            };
        }

        Ok(true)
    }
}
