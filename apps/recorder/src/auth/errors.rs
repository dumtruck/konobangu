use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error(transparent)]
    OidcInitError(#[from] jwt_authorizer::error::InitError),
    #[error("Invalid credentials")]
    BasicInvalidCredentials,
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

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(self.to_string())).into_response()
    }
}
