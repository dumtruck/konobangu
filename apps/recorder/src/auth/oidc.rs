use std::collections::{HashMap, HashSet};

use axum::http::request::Parts;
use itertools::Itertools;
use jwt_authorizer::{authorizer::Authorizer, NumericDate, OneOrArray};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    config::OidcAuthConfig,
    errors::AuthError,
    service::{AuthService, AuthUserInfo},
};
use crate::models::auth::AuthType;

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

pub struct OidcAuthService {
    pub config: OidcAuthConfig,
    pub authorizer: Authorizer<OidcAuthClaims>,
}

#[async_trait::async_trait]
impl AuthService for OidcAuthService {
    async fn extract_user_info(&self, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
        let config = &self.config;
        let token =
            self.authorizer
                .extract_token(&request.headers)
                .ok_or(AuthError::OidcJwtAuthError(
                    jwt_authorizer::AuthError::MissingToken(),
                ))?;

        let token_data = self.authorizer.check_auth(&token).await?;
        let claims = token_data.claims;
        if claims.sub.as_deref().is_none_or(|s| s.trim().is_empty()) {
            return Err(AuthError::OidcSubMissingError);
        }
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
        Ok(AuthUserInfo {
            user_pid: claims
                .sub
                .as_deref()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| unreachable!("sub should be present and validated")),
            auth_type: AuthType::Oidc,
        })
    }
}
