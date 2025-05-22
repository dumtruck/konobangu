use async_graphql::dynamic::ResolverContext;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fetch::HttpClientError;
use openidconnect::{
    ConfigurationError, RequestTokenError, SignatureVerificationError, SigningError,
    StandardErrorResponse, core::CoreErrorResponseType,
};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use util::OptDynErr;

use crate::models::auth::AuthType;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AuthError {
    #[snafu(display("Not support auth method"))]
    NotSupportAuthMethod {
        supported: Vec<AuthType>,
        current: AuthType,
    },
    #[snafu(display("Failed to find auth record"))]
    FindAuthRecordError,
    #[snafu(display("Invalid credentials"))]
    BasicInvalidCredentials,
    #[snafu(display("Invalid oidc provider meta client error: {source}"))]
    OidcProviderHttpClientError { source: HttpClientError },
    #[snafu(transparent)]
    OidcProviderMetaError {
        source: openidconnect::DiscoveryError<HttpClientError>,
    },
    #[snafu(display("Invalid oidc provider URL: {source}"))]
    OidcProviderUrlError { source: url::ParseError },
    #[snafu(display("Invalid oidc redirect URI: {source}"))]
    OidcRequestRedirectUriError {
        #[snafu(source)]
        source: url::ParseError,
    },
    #[snafu(display("Oidc request session not found or expired"))]
    OidcCallbackRecordNotFoundOrExpiredError,
    #[snafu(display("Invalid oidc request callback nonce"))]
    OidcInvalidNonceError,
    #[snafu(display("Invalid oidc request callback state"))]
    OidcInvalidStateError,
    #[snafu(display("Invalid oidc request callback code"))]
    OidcInvalidCodeError,
    #[snafu(transparent)]
    OidcCallbackTokenConfigurationError { source: ConfigurationError },
    #[snafu(transparent)]
    OidcRequestTokenError {
        source: RequestTokenError<HttpClientError, StandardErrorResponse<CoreErrorResponseType>>,
    },
    #[snafu(display("Invalid oidc id token"))]
    OidcInvalidIdTokenError,
    #[snafu(display("Invalid oidc access token"))]
    OidcInvalidAccessTokenError,
    #[snafu(transparent)]
    OidcSignatureVerificationError { source: SignatureVerificationError },
    #[snafu(transparent)]
    OidcSigningError { source: SigningError },
    #[snafu(display("Missing Bearer token"))]
    OidcMissingBearerToken,
    #[snafu(transparent)]
    OidcJwtkError { source: jwtk::Error },
    #[snafu(display("Extra scopes {expected} do not match found scopes {found}"))]
    OidcExtraScopesMatchError { expected: String, found: String },
    #[snafu(display("Extra claim {key} does not match expected value {expected}, found {found}"))]
    OidcExtraClaimMatchError {
        key: String,
        expected: String,
        found: String,
    },
    #[snafu(display("Extra claim {claim} missing"))]
    OidcExtraClaimMissingError { claim: String },
    #[snafu(display("Audience {aud} missing"))]
    OidcAudMissingError { aud: String },
    #[snafu(display("Subject missing"))]
    OidcSubMissingError,
    #[snafu(display(
        "GraphQL permission denied since {context_path}{}{field}{}{column}: {}",
        (if field.is_empty() { "" } else { "." }),
        (if column.is_empty() { "" } else { "." }),
        source.message
    ))]
    GraphqlDynamicPermissionError {
        #[snafu(source(false))]
        source: Box<async_graphql::Error>,
        field: String,
        column: String,
        context_path: String,
    },
    #[snafu(display("GraphQL permission denied since {field}"))]
    GraphqlStaticPermissionError {
        #[snafu(source)]
        source: OptDynErr,
        field: String,
    },
}

impl AuthError {
    pub fn from_graphql_dynamic_subscribe_id_guard(
        source: async_graphql::Error,
        context: &ResolverContext,
        field_name: &str,
        column_name: &str,
    ) -> AuthError {
        AuthError::GraphqlDynamicPermissionError {
            source: Box::new(source),
            field: field_name.to_string(),
            column: column_name.to_string(),
            context_path: context
                .ctx
                .path_node
                .map(|p| p.to_string_vec().join(""))
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthErrorResponse {
    pub success: bool,
    pub message: String,
}

impl From<AuthError> for AuthErrorResponse {
    fn from(value: AuthError) -> Self {
        AuthErrorResponse {
            success: false,
            message: value.to_string(),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse::from(self)),
        )
            .into_response()
    }
}
