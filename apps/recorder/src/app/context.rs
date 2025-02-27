use sea_orm::DatabaseConnection;

use super::config::AppConfig;
use crate::{
    auth::AuthService, cache::CacheService, errors::RResult, extract::mikan::MikanClient,
    graphql::GraphQLService, storage::StorageService,
};

pub struct AppContext {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub cache: CacheService,
    pub mikan: MikanClient,
    pub auth: AuthService,
    pub graphql: GraphQLService,
    pub storage: StorageService,
    pub working_dir: String,
}

pub async fn create_context(_config: AppConfig) -> RResult<AppContext> {
    todo!()
}
