pub mod ext;

use std::{
    fs,
    path::{self, Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
pub use ext::AppContextExt;
use itertools::Itertools;
use loco_rs::{
    Result,
    app::{AppContext, Hooks},
    boot::{BootResult, StartMode, create_app},
    cache,
    config::Config,
    controller::{AppRoutes, middleware, middleware::MiddlewareLayer},
    db::truncate_table,
    environment::Environment,
    prelude::*,
    task::Tasks,
};
use once_cell::sync::OnceCell;

use crate::{
    auth::service::AppAuthServiceInitializer,
    controllers::{self},
    dal::AppDalInitalizer,
    extract::mikan::client::AppMikanClientInitializer,
    graphql::service::AppGraphQLServiceInitializer,
    migrations::Migrator,
    models::subscribers,
    workers::subscription_worker::SubscriptionWorker,
};

pub const WORKING_ROOT_VAR_NAME: &str = "WORKING_ROOT";

static APP_WORKING_ROOT: OnceCell<quirks_path::PathBuf> = OnceCell::new();

pub struct App;

impl App {
    pub fn set_working_root(path: PathBuf) {
        APP_WORKING_ROOT.get_or_init(|| {
            quirks_path::PathBuf::from(path.as_os_str().to_string_lossy().to_string())
        });
    }

    pub fn get_working_root() -> &'static quirks_path::Path {
        APP_WORKING_ROOT
            .get()
            .map(|p| p.as_path())
            .expect("working root not set")
    }
}

#[async_trait]
impl Hooks for App {
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

    async fn load_config(env: &Environment) -> Result<Config> {
        let working_roots_to_search = [
            std::env::var(WORKING_ROOT_VAR_NAME).ok(),
            Some(String::from("./apps/recorder")),
            Some(String::from(".")),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        for working_root in working_roots_to_search.iter() {
            let working_root = PathBuf::from(working_root);
            for env_file in [
                working_root.join(format!(".env.{env}.local")),
                working_root.join(format!(".env.{env}")),
                working_root.join(".env.local"),
                working_root.join(".env"),
            ] {
                tracing::info!(env_file =? env_file);
                if env_file.exists() && env_file.is_file() {
                    dotenv::from_path(&env_file).map_err(loco_rs::Error::wrap)?;
                    tracing::info!("loaded env from {} success.", env_file.to_string_lossy());
                }
            }
        }

        for working_root in working_roots_to_search.iter() {
            let working_root = PathBuf::from(working_root);
            let config_dir = working_root.as_path().join("config");

            for config_file in [
                config_dir.join(format!("{env}.local.yaml")),
                config_dir.join(format!("{env}.yaml")),
            ] {
                if config_file.exists() && config_file.is_file() {
                    let content = fs::read_to_string(config_file.clone())?;

                    let rendered = tera::Tera::one_off(
                        &content,
                        &tera::Context::from_value(serde_json::json!({}))?,
                        false,
                    )?;

                    App::set_working_root(working_root);

                    let config_file = &config_file.to_string_lossy();

                    let res = serde_yaml::from_str(&rendered)
                        .map_err(|err| loco_rs::Error::YAMLFile(err, config_file.to_string()))?;

                    tracing::info!("loading config from {} success", config_file);

                    return Ok(res);
                }
            }
        }

        Err(loco_rs::Error::Message(format!(
            "no configuration file found in search paths: {}",
            working_roots_to_search
                .iter()
                .flat_map(|p| path::absolute(PathBuf::from(p)))
                .map(|p| p.to_string_lossy().to_string())
                .join(",")
        )))
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        let initializers: Vec<Box<dyn Initializer>> = vec![
            Box::new(AppDalInitalizer),
            Box::new(AppMikanClientInitializer),
            Box::new(AppGraphQLServiceInitializer),
            Box::new(AppAuthServiceInitializer),
        ];

        Ok(initializers)
    }

    fn routes(ctx: &AppContext) -> AppRoutes {
        let ctx = Arc::new(ctx.clone());
        AppRoutes::with_default_routes()
            .prefix("/api")
            .add_route(controllers::graphql::routes(ctx.clone()))
    }

    fn middlewares(ctx: &AppContext) -> Vec<Box<dyn MiddlewareLayer>> {
        use loco_rs::controller::middleware::static_assets::{FolderConfig, StaticAssets};

        let mut middlewares = middleware::default_middleware_stack(ctx);
        middlewares.push(Box::new(StaticAssets {
            enable: true,
            must_exist: true,
            folder: FolderConfig {
                uri: String::from("/api/static"),
                path: App::get_working_root().join("public").into(),
            },
            fallback: App::get_working_root()
                .join("public/assets/404.html")
                .into(),
            precompressed: false,
        }));
        middlewares
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        Ok(AppContext {
            cache: cache::Cache::new(cache::drivers::inmem::new()).into(),
            ..ctx
        })
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(SubscriptionWorker::build(ctx)).await?;
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, subscribers::Entity).await?;
        Ok(())
    }

    async fn seed(_ctx: &AppContext, _base: &Path) -> Result<()> {
        Ok(())
    }
}
