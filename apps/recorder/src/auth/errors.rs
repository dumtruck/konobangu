use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    BasicInvalidCredentials,
    #[error(transparent)]
    OidcInitError(#[from] jwt_authorizer::error::InitError),
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
