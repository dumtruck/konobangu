use typed_builder::TypedBuilder;

use crate::app::AppContextTrait;

#[derive(TypedBuilder)]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct UnitTestAppContext {
    logger: Option<crate::logger::LoggerService>,
    db: Option<crate::database::DatabaseService>,
    config: Option<crate::app::AppConfig>,
    cache: Option<crate::cache::CacheService>,
    mikan: Option<crate::extract::mikan::MikanClient>,
    auth: Option<crate::auth::AuthService>,
    graphql: Option<crate::graphql::GraphQLService>,
    storage: Option<crate::storage::StorageService>,
    #[builder(default = Some(String::from(env!("CARGO_MANIFEST_DIR"))))]
    working_dir: Option<String>,
    #[builder(default = crate::app::Environment::Testing, setter(!strip_option))]
    environment: crate::app::Environment,
}

impl AppContextTrait for UnitTestAppContext {
    fn logger(&self) -> &crate::logger::LoggerService {
        self.logger.as_ref().expect("should set logger")
    }

    fn db(&self) -> &crate::database::DatabaseService {
        self.db.as_ref().expect("should set db")
    }

    fn config(&self) -> &crate::app::AppConfig {
        self.config.as_ref().expect("should set config")
    }

    fn cache(&self) -> &crate::cache::CacheService {
        self.cache.as_ref().expect("should set cache")
    }

    fn mikan(&self) -> &crate::extract::mikan::MikanClient {
        self.mikan.as_ref().expect("should set mikan")
    }

    fn auth(&self) -> &crate::auth::AuthService {
        self.auth.as_ref().expect("should set auth")
    }

    fn graphql(&self) -> &crate::graphql::GraphQLService {
        self.graphql.as_ref().expect("should set graphql")
    }

    fn storage(&self) -> &crate::storage::StorageService {
        self.storage.as_ref().expect("should set storage")
    }

    fn environment(&self) -> &crate::app::Environment {
        &self.environment
    }

    fn working_dir(&self) -> &String {
        self.working_dir.as_ref().expect("should set working_dir")
    }
}
