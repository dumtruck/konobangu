use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use openidconnect::{
    core::CoreErrorResponseType, ConfigurationError, RequestTokenError, SignatureVerificationError,
    SigningError, StandardErrorResponse,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{fetch::HttpClientError, models::auth::AuthType};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Not support auth method")]
    NotSupportAuthMethod {
        supported: Vec<AuthType>,
        current: AuthType,
    },
    #[error("Invalid credentials")]
    BasicInvalidCredentials,
    #[error(transparent)]
    OidcInitError(#[from] jwt_authorizer::error::InitError),
    #[error("Invalid oidc provider meta client error: {0}")]
    OidcProviderHttpClientError(HttpClientError),
    #[error(transparent)]
    OidcProviderMetaError(#[from] openidconnect::DiscoveryError<HttpClientError>),
    #[error("Invalid oidc provider URL: {0}")]
    OidcProviderUrlError(url::ParseError),
    #[error("Invalid oidc redirect URI: {0}")]
    OidcRequestRedirectUriError(url::ParseError),
    #[error("Oidc request session not found or expired")]
    OidcCallbackRecordNotFoundOrExpiredError,
    #[error("Invalid oidc request callback nonce")]
    OidcInvalidNonceError,
    #[error("Invalid oidc request callback state")]
    OidcInvalidStateError,
    #[error("Invalid oidc request callback code")]
    OidcInvalidCodeError,
    #[error(transparent)]
    OidcCallbackTokenConfigrationError(#[from] ConfigurationError),
    #[error(transparent)]
    OidcRequestTokenError(
        #[from] RequestTokenError<HttpClientError, StandardErrorResponse<CoreErrorResponseType>>,
    ),
    #[error("Invalid oidc id token")]
    OidcInvalidIdTokenError,
    #[error("Invalid oidc access token")]
    OidcInvalidAccessTokenError,
    #[error(transparent)]
    OidcSignatureVerificationError(#[from] SignatureVerificationError),
    #[error(transparent)]
    OidcSigningError(#[from] SigningError),
    #[error(transparent)]
    OidcJwtAuthError(#[from] jwt_authorizer::AuthError),
    #[error("Extra scopes {expected} do not match found scopes {found}")]
    OidcExtraScopesMatchError { expected: String, found: String },
    #[error("Extra claim {key} does not match expected value {expected}, found {found}")]
    OidcExtraClaimMatchError {
        key: String,
        expected: String,
        found: String,
    },
    #[error("Extra claim {0} missing")]
    OidcExtraClaimMissingError(String),
    #[error("Audience {0} missing")]
    OidcAudMissingError(String),
    #[error("Subject missing")]
    OidcSubMissingError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthErrorBody {
    pub error_code: i32,
    pub error_msg: String,
}

impl From<AuthError> for AuthErrorBody {
    fn from(value: AuthError) -> Self {
        AuthErrorBody {
            error_code: StatusCode::UNAUTHORIZED.as_u16() as i32,
            error_msg: value.to_string(),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(AuthErrorBody::from(self))).into_response()
    }
}
