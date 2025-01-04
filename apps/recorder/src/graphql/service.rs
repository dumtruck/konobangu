use async_graphql::dynamic::{Schema, SchemaError};
use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use once_cell::sync::OnceCell;
use sea_orm::DatabaseConnection;

use super::{config::AppGraphQLConfig, query_root};
use crate::config::AppConfigExt;

static APP_GRAPHQL_SERVICE: OnceCell<AppGraphQLService> = OnceCell::new();

#[derive(Debug)]
pub struct AppGraphQLService {
    pub schema: Schema,
}

impl AppGraphQLService {
    pub fn new(config: AppGraphQLConfig, db: DatabaseConnection) -> Result<Self, SchemaError> {
        let schema = query_root::schema(db, config.depth_limit, config.complexity_limit)?;
        Ok(Self { schema })
    }

    pub fn app_instance() -> &'static Self {
        APP_GRAPHQL_SERVICE
            .get()
            .expect("AppGraphQLService is not initialized")
    }
}

#[derive(Debug, Clone)]
pub struct AppGraphQLServiceInitializer;

#[async_trait]
impl Initializer for AppGraphQLServiceInitializer {
    fn name(&self) -> String {
        String::from("AppGraphQLServiceInitializer")
    }

    async fn before_run(&self, app_context: &AppContext) -> loco_rs::Result<()> {
        APP_GRAPHQL_SERVICE.get_or_try_init(|| {
            let config = app_context
                .config
                .get_app_conf()
                .map_err(loco_rs::Error::wrap)?
                .graphql;
            let db = &app_context.db;
            AppGraphQLService::new(config, db.clone()).map_err(loco_rs::Error::wrap)
        })?;
        Ok(())
    }
}
