use super::{Environment, config::AppConfig};
use crate::{
    auth::AuthService, cache::CacheService, database::DatabaseService, errors::RResult,
    extract::mikan::MikanClient, graphql::GraphQLService, logger::LoggerService,
    storage::StorageService,
};

pub trait AppContextTrait: Send + Sync {
    fn logger(&self) -> &LoggerService;
    fn db(&self) -> &DatabaseService;
    fn config(&self) -> &AppConfig;
    fn cache(&self) -> &CacheService;
    fn mikan(&self) -> &MikanClient;
    fn auth(&self) -> &AuthService;
    fn graphql(&self) -> &GraphQLService;
    fn storage(&self) -> &StorageService;
    fn working_dir(&self) -> &String;
    fn environment(&self) -> &Environment;
}

pub struct AppContext {
    logger: LoggerService,
    db: DatabaseService,
    config: AppConfig,
    cache: CacheService,
    mikan: MikanClient,
    auth: AuthService,
    graphql: GraphQLService,
    storage: StorageService,
    working_dir: String,
    environment: Environment,
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
impl AppContextTrait for AppContext {
    fn logger(&self) -> &LoggerService {
        &self.logger
    }
    fn db(&self) -> &DatabaseService {
        &self.db
    }
    fn config(&self) -> &AppConfig {
        &self.config
    }
    fn cache(&self) -> &CacheService {
        &self.cache
    }
    fn mikan(&self) -> &MikanClient {
        &self.mikan
    }
    fn auth(&self) -> &AuthService {
        &self.auth
    }
    fn graphql(&self) -> &GraphQLService {
        &self.graphql
    }
    fn storage(&self) -> &StorageService {
        &self.storage
    }
    fn working_dir(&self) -> &String {
        &self.working_dir
    }
    fn environment(&self) -> &Environment {
        &self.environment
    }
}
