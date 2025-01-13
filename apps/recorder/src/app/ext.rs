use loco_rs::{app::AppContext, environment::Environment};

use crate::{
    auth::service::AppAuthService, dal::AppDalClient, extract::mikan::AppMikanClient,
    graphql::service::AppGraphQLService,
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

    fn get_graphql_service(&self) -> &AppGraphQLService {
        AppGraphQLService::app_instance()
    }

    fn get_node_env(&self) -> Environment {
        let node_env = std::env::var("NODE_ENV");
        match node_env.as_deref() {
            Ok("production") => Environment::Production,
            _ => Environment::Development,
        }
    }
}

impl AppContextExt for AppContext {}
