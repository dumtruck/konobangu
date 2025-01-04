use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse as _, Response},
    Extension,
};
use jwt_authorizer::{JwtAuthorizer, Validation};
use loco_rs::app::{AppContext, Initializer};
use once_cell::sync::OnceCell;

use super::{
    basic::BasicAuthService,
    errors::AuthError,
    oidc::{OidcAuthClaims, OidcAuthService},
    AppAuthConfig,
};
use crate::{app::AppContextExt as _, config::AppConfigExt, models::auth::AuthType};

pub struct AuthUserInfo {
    pub user_pid: String,
    pub auth_type: AuthType,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUserInfo
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(ctx) = Extension::<AppContext>::from_request_parts(req, state)
            .await
            .expect("AppContext should be present");

        let auth_service = ctx.get_auth_service();

        auth_service
            .extract_user_info(req)
            .await
            .map_err(|err| err.into_response())
    }
}

#[async_trait]
pub trait AuthService {
    async fn extract_user_info(&self, request: &mut Parts) -> Result<AuthUserInfo, AuthError>;
}

pub enum AppAuthService {
    Basic(BasicAuthService),
    Oidc(OidcAuthService),
}

static APP_AUTH_SERVICE: OnceCell<AppAuthService> = OnceCell::new();

impl AppAuthService {
    pub fn app_instance() -> &'static Self {
        APP_AUTH_SERVICE
            .get()
            .expect("AppAuthService is not initialized")
    }

    pub async fn from_conf(config: AppAuthConfig) -> Result<Self, AuthError> {
        let result = match config {
            AppAuthConfig::Basic(config) => AppAuthService::Basic(BasicAuthService { config }),
            AppAuthConfig::Oidc(config) => {
                let validation = Validation::new()
                    .iss(&[&config.issuer])
                    .aud(&[&config.audience]);

                let jwt_auth = JwtAuthorizer::<OidcAuthClaims>::from_oidc(&config.issuer)
                    .validation(validation)
                    .build()
                    .await?;

                AppAuthService::Oidc(OidcAuthService {
                    config,
                    authorizer: jwt_auth,
                })
            }
        };
        Ok(result)
    }
}

#[async_trait]
impl AuthService for AppAuthService {
    async fn extract_user_info(&self, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
        match self {
            AppAuthService::Basic(service) => service.extract_user_info(request).await,
            AppAuthService::Oidc(service) => service.extract_user_info(request).await,
        }
    }
}

pub struct AppAuthServiceInitializer;

#[async_trait]
impl Initializer for AppAuthServiceInitializer {
    fn name(&self) -> String {
        String::from("AppAuthServiceInitializer")
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<(), loco_rs::Error> {
        let auth_conf = ctx.config.get_app_conf()?.auth;

        let service = AppAuthService::from_conf(auth_conf)
            .await
            .map_err(loco_rs::Error::wrap)?;

        APP_AUTH_SERVICE.get_or_init(|| service);

        Ok(())
    }
}
