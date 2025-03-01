use super::{Environment, config::AppConfig};
use crate::{
    auth::AuthService, cache::CacheService, database::DatabaseService, errors::RResult,
    extract::mikan::MikanClient, graphql::GraphQLService, logger::LoggerService,
    storage::StorageService,
};

pub struct AppContext {
    pub logger: LoggerService,
    pub db: DatabaseService,
    pub config: AppConfig,
    pub cache: CacheService,
    pub mikan: MikanClient,
    pub auth: AuthService,
    pub graphql: GraphQLService,
    pub storage: StorageService,
    pub working_dir: String,
    pub environment: Environment,
}

impl AppContext {
    pub async fn new(
        environment: Environment,
        config: AppConfig,
        working_dir: impl ToString,
    ) -> RResult<Self> {
        let config_cloned = config.clone();

        let logger = LoggerService::from_config(config.logger).await?;
        let cache = CacheService::from_config(config.cache).await?;
        let db = DatabaseService::from_config(config.database).await?;
        let storage = StorageService::from_config(config.storage).await?;
        let auth = AuthService::from_conf(config.auth).await?;
        let mikan = MikanClient::from_config(config.mikan).await?;
        let graphql = GraphQLService::from_config_and_database(config.graphql, db.clone()).await?;

        Ok(AppContext {
            config: config_cloned,
            environment,
            logger,
            auth,
            cache,
            db,
            storage,
            mikan,
            working_dir: working_dir.to_string(),
            graphql,
        })
    }
}
