use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use axum::http::{HeaderValue, request::Parts};
use itertools::Itertools;
use jwt_authorizer::{NumericDate, OneOrArray, authorizer::Authorizer};
use moka::future::Cache;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
};
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::{
    config::OidcAuthConfig,
    errors::AuthError,
    service::{AuthServiceTrait, AuthUserInfo},
};
use crate::{app::AppContext, errors::RError, fetch::HttpClient, models::auth::AuthType};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OidcAuthClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<OneOrArray<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<NumericDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<NumericDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<NumericDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}

impl OidcAuthClaims {
    pub fn scopes(&self) -> std::str::Split<'_, char> {
        self.scope.as_deref().unwrap_or_default().split(',')
    }

    pub fn get_claim(&self, key: &str) -> Option<String> {
        match key {
            "iss" => self.iss.clone(),
            "sub" => self.sub.clone(),
            "aud" => self.aud.as_ref().map(|s| s.iter().join(",")),
            "exp" => self.exp.clone().map(|s| s.0.to_string()),
            "nbf" => self.nbf.clone().map(|s| s.0.to_string()),
            "iat" => self.iat.clone().map(|s| s.0.to_string()),
            "jti" => self.jti.clone(),
            "scope" => self.scope.clone(),
            key => self.custom.get(key).map(|s| s.to_string()),
        }
    }

    pub fn has_claim(&self, key: &str) -> bool {
        match key {
            "iss" => self.iss.is_some(),
            "sub" => self.sub.is_some(),
            "aud" => self.aud.is_some(),
            "exp" => self.exp.is_some(),
            "nbf" => self.nbf.is_some(),
            "iat" => self.iat.is_some(),
            "jti" => self.jti.is_some(),
            "scope" => self.scope.is_some(),
            key => self.custom.contains_key(key),
        }
    }

    pub fn contains_audience(&self, aud: &str) -> bool {
        self.aud
            .as_ref()
            .is_some_and(|arr| arr.iter().any(|s| s == aud))
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
    pub api_authorizer: Authorizer<OidcAuthClaims>,
    pub oidc_provider_client: HttpClient,
    pub oidc_request_cache: Cache<String, OidcAuthRequest>,
}

impl OidcAuthService {
    pub async fn build_authorization_request(
        &self,
        redirect_uri: &str,
    ) -> Result<OidcAuthRequest, AuthError> {
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(self.config.issuer.clone()).map_err(AuthError::OidcProviderUrlError)?,
            &self.oidc_provider_client,
        )
        .await?;

        let redirect_uri = RedirectUrl::new(redirect_uri.to_string())
            .map_err(AuthError::OidcRequestRedirectUriError)?;

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
        let csrf_token = query.state.ok_or(AuthError::OidcInvalidStateError)?;

        let code = query.code.ok_or(AuthError::OidcInvalidCodeError)?;

        let request_cache = self.load_authorization_request(&csrf_token).await?;

        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(self.config.issuer.clone()).map_err(AuthError::OidcProviderUrlError)?,
            &self.oidc_provider_client,
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
            .request_async(&HttpClient::default())
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

        if let Some(expected_access_token_hash) = claims.access_token_hash() {
            if actual_access_token_hash != *expected_access_token_hash {
                return Err(AuthError::OidcInvalidAccessTokenError);
            }
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
        ctx: &AppContext,
        request: &mut Parts,
    ) -> Result<AuthUserInfo, AuthError> {
        let config = &self.config;
        let token = self.api_authorizer.extract_token(&request.headers).ok_or(
            AuthError::OidcJwtAuthError(jwt_authorizer::AuthError::MissingToken()),
        )?;

        let token_data = self.api_authorizer.check_auth(&token).await?;
        let claims = token_data.claims;
        let sub = if let Some(sub) = claims.sub.as_deref() {
            sub
        } else {
            return Err(AuthError::OidcSubMissingError);
        };
        if !claims.contains_audience(&config.audience) {
            return Err(AuthError::OidcAudMissingError(config.audience.clone()));
        }
        if let Some(expected_scopes) = config.extra_scopes.as_ref() {
            let found_scopes = claims.scopes().collect::<HashSet<_>>();
            if !expected_scopes
                .iter()
                .all(|es| found_scopes.contains(es as &str))
            {
                return Err(AuthError::OidcExtraScopesMatchError {
                    expected: expected_scopes.iter().join(","),
                    found: claims.scope.unwrap_or_default(),
                });
            }
        }
        if let Some(key) = config.extra_claim_key.as_ref() {
            if !claims.has_claim(key) {
                return Err(AuthError::OidcExtraClaimMissingError(key.clone()));
            }
            if let Some(value) = config.extra_claim_value.as_ref() {
                if claims.get_claim(key).is_none_or(|v| &v != value) {
                    return Err(AuthError::OidcExtraClaimMatchError {
                        expected: value.clone(),
                        found: claims.get_claim(key).unwrap_or_default().to_string(),
                        key: key.clone(),
                    });
                }
            }
        }
        let subscriber_auth = match crate::models::auth::Model::find_by_pid(ctx, sub).await {
            Err(RError::DbError(DbErr::RecordNotFound(..))) => {
                crate::models::auth::Model::create_from_oidc(ctx, sub.to_string()).await
            }
            r => r,
        }
        .map_err(|_| AuthError::FindAuthRecordError)?;

        Ok(AuthUserInfo {
            subscriber_auth,
            auth_type: AuthType::Oidc,
        })
    }

    fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
        Some(HeaderValue::from_static(r#"Bearer realm="konobangu""#))
    }

    fn auth_type(&self) -> AuthType {
        AuthType::Oidc
    }
}
