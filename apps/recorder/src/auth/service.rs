use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse as _, Response},
};
use fetch::{
    HttpClient, HttpClientConfig,
    client::{HttpClientCacheBackendConfig, HttpClientCachePresetConfig},
};
use http::header::HeaderValue;
use jwt_authorizer::{JwtAuthorizer, Validation};
use moka::future::Cache;
use snafu::prelude::*;

use super::{
    AuthConfig,
    basic::BasicAuthService,
    errors::{AuthError, OidcProviderHttpClientSnafu},
    oidc::{OidcAuthClaims, OidcAuthService},
};
use crate::{app::AppContextTrait, models::auth::AuthType};

#[derive(Clone, Debug)]
pub struct AuthUserInfo {
    pub subscriber_auth: crate::models::auth::Model,
    pub auth_type: AuthType,
}

impl FromRequestParts<Arc<dyn AppContextTrait>> for AuthUserInfo {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<dyn AppContextTrait>,
    ) -> Result<Self, Self::Rejection> {
        let auth_service = state.auth();

        auth_service
            .extract_user_info(state.as_ref(), parts)
            .await
            .map_err(|err| err.into_response())
    }
}

#[async_trait]
pub trait AuthServiceTrait {
    async fn extract_user_info(
        &self,
        ctx: &dyn AppContextTrait,
        request: &mut Parts,
    ) -> Result<AuthUserInfo, AuthError>;
    fn www_authenticate_header_value(&self) -> Option<HeaderValue>;
    fn auth_type(&self) -> AuthType;
}

pub enum AuthService {
    Basic(Box<BasicAuthService>),
    Oidc(Box<OidcAuthService>),
}

impl AuthService {
    pub async fn from_conf(config: AuthConfig) -> Result<Self, AuthError> {
        let result = match config {
            AuthConfig::Basic(config) => AuthService::Basic(Box::new(BasicAuthService { config })),
            AuthConfig::Oidc(config) => {
                let validation = Validation::new()
                    .iss(&[&config.issuer])
                    .aud(&[&config.audience]);

                let oidc_provider_client = HttpClient::from_config(HttpClientConfig {
                    exponential_backoff_max_retries: Some(3),
                    cache_backend: Some(HttpClientCacheBackendConfig::Moka { cache_size: 1 }),
                    cache_preset: Some(HttpClientCachePresetConfig::RFC7234),
                    ..Default::default()
                })
                .context(OidcProviderHttpClientSnafu)?;

                let api_authorizer = JwtAuthorizer::<OidcAuthClaims>::from_oidc(&config.issuer)
                    .validation(validation)
                    .build()
                    .await?;

                AuthService::Oidc(Box::new(OidcAuthService {
                    config,
                    api_authorizer,
                    oidc_provider_client: Arc::new(oidc_provider_client),
                    oidc_request_cache: Cache::builder()
                        .time_to_live(Duration::from_mins(5))
                        .name("oidc_request_cache")
                        .build(),
                }))
            }
        };
        Ok(result)
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn extract_user_info(
        &self,
        ctx: &dyn AppContextTrait,
        request: &mut Parts,
    ) -> Result<AuthUserInfo, AuthError> {
        match self {
            AuthService::Basic(service) => service.extract_user_info(ctx, request).await,
            AuthService::Oidc(service) => service.extract_user_info(ctx, request).await,
        }
    }

    fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
        match self {
            AuthService::Basic(service) => service.www_authenticate_header_value(),
            AuthService::Oidc(service) => service.www_authenticate_header_value(),
        }
    }

    fn auth_type(&self) -> AuthType {
        match self {
            AuthService::Basic(service) => service.auth_type(),
            AuthService::Oidc(service) => service.auth_type(),
        }
    }
}
