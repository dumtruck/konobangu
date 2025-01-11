use loco_rs::app::AppContext;

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
}

impl AppContextExt for AppContext {}
