use std::{
    collections::{HashMap, HashSet},
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
};

use async_trait::async_trait;
use axum::{
    http,
    http::{HeaderValue, request::Parts},
};
use fetch::{HttpClient, client::HttpClientError};
use http::header::AUTHORIZATION;
use itertools::Itertools;
use jwtk::jwk::RemoteJwksVerifier;
use moka::future::Cache;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;
use url::Url;

use super::{
    config::OidcAuthConfig,
    errors::{AuthError, OidcProviderUrlSnafu, OidcRequestRedirectUriSnafu},
    service::{AuthServiceTrait, AuthUserInfo},
};
use crate::{
    app::{AppContextTrait, PROJECT_NAME},
    errors::RecorderError,
    models::auth::AuthType,
};

pub struct OidcHttpClient(pub Arc<HttpClient>);

impl Deref for OidcHttpClient {
    type Target = HttpClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'c> openidconnect::AsyncHttpClient<'c> for OidcHttpClient {
    type Error = HttpClientError;

    #[cfg(target_arch = "wasm32")]
    type Future =
        Pin<Box<dyn Future<Output = Result<openidconnect::HttpResponse, Self::Error>> + 'c>>;
    #[cfg(not(target_arch = "wasm32"))]
    type Future =
        Pin<Box<dyn Future<Output = Result<openidconnect::HttpResponse, Self::Error>> + Send + 'c>>;

    fn call(&'c self, request: openidconnect::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let response = self.execute(request.try_into()?).await?;

            let mut builder = http::Response::builder().status(response.status());

            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder.version(response.version());
            }

            for (name, value) in response.headers().iter() {
                builder = builder.header(name, value);
            }

            builder
                .body(response.bytes().await?.to_vec())
                .map_err(HttpClientError::from)
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OidcAuthClaims {
    pub scope: Option<String>,
    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}

impl OidcAuthClaims {
    pub fn scopes(&self) -> std::str::Split<'_, char> {
        self.scope.as_deref().unwrap_or_default().split(',')
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OidcAuthRequest {
    pub auth_uri: Url,
    #[serde(skip)]
    pub redirect_uri: RedirectUrl,
    #[serde(skip)]
    pub csrf_token: CsrfToken,
    #[serde(skip)]
    pub nonce: Nonce,
    #[serde(skip)]
    pub pkce_verifier: Arc<PkceCodeVerifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcAuthCallbackQuery {
    pub state: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcAuthCallbackPayload {
    pub access_token: String,
}

pub struct OidcAuthService {
    pub config: OidcAuthConfig,
    pub jwk_verifier: RemoteJwksVerifier,
    pub oidc_provider_client: Arc<HttpClient>,
    pub oidc_request_cache: Cache<String, OidcAuthRequest>,
}

impl OidcAuthService {
    pub async fn build_authorization_request(
        &self,
        redirect_uri: &str,
    ) -> Result<OidcAuthRequest, AuthError> {
        let oidc_provider_client = OidcHttpClient(self.oidc_provider_client.clone());
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(self.config.issuer.clone()).context(OidcProviderUrlSnafu)?,
            &oidc_provider_client,
        )
        .await?;

        let redirect_uri =
            RedirectUrl::new(redirect_uri.to_string()).context(OidcRequestRedirectUriSnafu)?;

        let oidc_client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(self.config.client_id.clone()),
            Some(ClientSecret::new(self.config.client_secret.clone())),
        )
        .set_redirect_uri(redirect_uri.clone());

        let (pkce_chanllenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut authorization_request = oidc_client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(pkce_chanllenge);

        {
            if let Some(scopes) = self.config.extra_scopes.as_ref() {
                authorization_request = authorization_request.add_scopes(
                    scopes
                        .iter()
                        .map(|s| openidconnect::Scope::new(s.to_string())),
                )
            }
        }

        let (auth_uri, csrf_token, nonce) = authorization_request.url();

        Ok(OidcAuthRequest {
            auth_uri,
            csrf_token,
            nonce,
            pkce_verifier: Arc::new(pkce_verifier),
            redirect_uri,
        })
    }

    pub async fn store_authorization_request(
        &self,
        request: OidcAuthRequest,
    ) -> Result<(), AuthError> {
        self.oidc_request_cache
            .insert(request.csrf_token.secret().to_string(), request)
            .await;
        Ok(())
    }

    pub async fn load_authorization_request(
        &self,
        state: &str,
    ) -> Result<OidcAuthRequest, AuthError> {
        let result = self
            .oidc_request_cache
            .get(state)
            .await
            .ok_or(AuthError::OidcCallbackRecordNotFoundOrExpiredError)?;

        self.oidc_request_cache.invalidate(state).await;

        Ok(result)
    }

    pub async fn extract_authorization_request_callback(
        &self,
        query: OidcAuthCallbackQuery,
    ) -> Result<OidcAuthCallbackPayload, AuthError> {
        let oidc_http_client = OidcHttpClient(self.oidc_provider_client.clone());
        let csrf_token = query.state.ok_or(AuthError::OidcInvalidStateError)?;

        let code = query.code.ok_or(AuthError::OidcInvalidCodeError)?;

        let request_cache = self.load_authorization_request(&csrf_token).await?;

        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(self.config.issuer.clone()).context(OidcProviderUrlSnafu)?,
            &oidc_http_client,
        )
        .await?;

        let oidc_client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(self.config.client_id.clone()),
            Some(ClientSecret::new(self.config.client_secret.clone())),
        )
        .set_redirect_uri(request_cache.redirect_uri);

        let pkce_verifier = PkceCodeVerifier::new(request_cache.pkce_verifier.secret().to_string());

        let token_response = oidc_client
            .exchange_code(AuthorizationCode::new(code))?
            .set_pkce_verifier(pkce_verifier)
            .request_async(&oidc_http_client)
            .await?;

        let id_token = token_response
            .id_token()
            .ok_or(AuthError::OidcInvalidIdTokenError)?;

        let id_token_verifier = &oidc_client.id_token_verifier();

        let claims = id_token
            .claims(id_token_verifier, &request_cache.nonce)
            .map_err(|_| AuthError::OidcInvalidNonceError)?;

        let access_token = token_response.access_token();

        let actual_access_token_hash = AccessTokenHash::from_token(
            access_token,
            id_token.signing_alg()?,
            id_token.signing_key(id_token_verifier)?,
        )?;

        if let Some(expected_access_token_hash) = claims.access_token_hash()
            && actual_access_token_hash != *expected_access_token_hash
        {
            return Err(AuthError::OidcInvalidAccessTokenError);
        }

        Ok(OidcAuthCallbackPayload {
            access_token: access_token.secret().to_string(),
        })
    }
}

#[async_trait]
impl AuthServiceTrait for OidcAuthService {
    async fn extract_user_info(
        &self,
        ctx: &dyn AppContextTrait,
        request: &mut Parts,
    ) -> Result<AuthUserInfo, AuthError> {
        let config = &self.config;
        let token = request
            .headers
            .get(AUTHORIZATION)
            .and_then(|authorization| {
                authorization
                    .to_str()
                    .ok()
                    .and_then(|s| s.strip_prefix("Bearer "))
            })
            .ok_or(AuthError::OidcMissingBearerToken)?;

        let token_data = self.jwk_verifier.verify::<OidcAuthClaims>(token).await?;
        let claims = token_data.claims();
        let sub = if let Some(sub) = claims.sub.as_deref() {
            sub
        } else {
            return Err(AuthError::OidcSubMissingError);
        };
        if !claims.aud.iter().any(|aud| aud == &config.audience) {
            return Err(AuthError::OidcAudMissingError {
                aud: config.audience.clone(),
            });
        }
        let extra_claims = &claims.extra;
        if let Some(expected_scopes) = config.extra_scopes.as_ref() {
            let found_scopes = extra_claims.scopes().collect::<HashSet<_>>();
            if !expected_scopes
                .iter()
                .all(|es| found_scopes.contains(es as &str))
            {
                return Err(AuthError::OidcExtraScopesMatchError {
                    expected: expected_scopes.iter().join(","),
                    found: extra_claims
                        .scope
                        .as_deref()
                        .unwrap_or_default()
                        .to_string(),
                });
            }
        }
        if let Some(expected_extra_claims) = config.extra_claims.as_ref() {
            for (expected_key, expected_value) in expected_extra_claims.iter() {
                match (extra_claims.custom.get(expected_key), expected_value) {
                    (found_value, Some(expected_value)) => {
                        if let Some(Value::String(found_value)) = found_value
                            && expected_value == found_value
                        {
                        } else {
                            return Err(AuthError::OidcExtraClaimMatchError {
                                expected: expected_value.clone(),
                                found: found_value.map(|v| v.to_string()).unwrap_or_default(),
                                key: expected_key.clone(),
                            });
                        }
                    }
                    (None, None) => {
                        return Err(AuthError::OidcExtraClaimMissingError {
                            claim: expected_key.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }
        let subscriber_auth = match crate::models::auth::Model::find_by_pid(ctx, sub).await {
            Err(RecorderError::DbError {
                source: DbErr::RecordNotFound(..),
            }) => crate::models::auth::Model::create_from_oidc(ctx, sub.to_string()).await,
            r => r,
        }
        .map_err(|e| {
            tracing::error!("Error finding auth record: {:?}", e);
            AuthError::FindAuthRecordError
        })?;

        Ok(AuthUserInfo {
            subscriber_auth,
            auth_type: AuthType::Oidc,
        })
    }

    fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
        Some(HeaderValue::from_str(format!("Bearer realm=\"{PROJECT_NAME}\"").as_str()).unwrap())
    }

    fn auth_type(&self) -> AuthType {
        AuthType::Oidc
    }
}
