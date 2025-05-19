use std::{fmt::Debug, sync::Arc};

use once_cell::sync::OnceCell;
use typed_builder::TypedBuilder;

use crate::app::AppContextTrait;

#[derive(TypedBuilder)]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct TestingAppContext {
    logger: Option<crate::logger::LoggerService>,
    db: Option<crate::database::DatabaseService>,
    config: Option<crate::app::AppConfig>,
    cache: Option<crate::cache::CacheService>,
    mikan: Option<crate::extract::mikan::MikanClient>,
    auth: Option<crate::auth::AuthService>,
    graphql: Option<crate::graphql::GraphQLService>,
    storage: Option<crate::storage::StorageService>,
    crypto: Option<crate::crypto::CryptoService>,
    #[builder(default = Arc::new(OnceCell::new()), setter(!strip_option))]
    task: Arc<OnceCell<crate::task::TaskService>>,
    message: Option<crate::message::MessageService>,
    #[builder(default = Some(String::from(env!("CARGO_MANIFEST_DIR"))))]
    working_dir: Option<String>,
    #[builder(default = crate::app::Environment::Testing, setter(!strip_option))]
    environment: crate::app::Environment,
}

impl TestingAppContext {
    pub fn set_task(&self, task: crate::task::TaskService) {
        self.task.get_or_init(|| task);
    }
}

impl Debug for TestingAppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnitTestAppContext")
    }
}

impl AppContextTrait for TestingAppContext {
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

    fn storage(&self) -> &dyn crate::storage::StorageServiceTrait {
        self.storage.as_ref().expect("should set storage")
    }

    fn environment(&self) -> &crate::app::Environment {
        &self.environment
    }

    fn working_dir(&self) -> &String {
        self.working_dir.as_ref().expect("should set working_dir")
    }

    fn crypto(&self) -> &crate::crypto::CryptoService {
        self.crypto.as_ref().expect("should set crypto")
    }

    fn task(&self) -> &crate::task::TaskService {
        self.task.get().expect("should set task")
    }

    fn message(&self) -> &crate::message::MessageService {
        self.message.as_ref().expect("should set message")
    }
}
