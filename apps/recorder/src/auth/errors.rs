use async_graphql::dynamic::ResolverContext;
use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AuthError {
  #[snafu(display("Authentication configuration is invalid: {message}"))]
  Configuration { message: String },
  #[snafu(display("Identity storage is unavailable"))]
  IdentityStorage,
  #[snafu(display("Authentication is required"))]
  Unauthorized,
  #[snafu(display("A bearer access token is required"))]
  ResourceTokenMissing,
  #[snafu(display("The access token is not valid for this API"))]
  ResourceTokenInvalid,
  #[snafu(display("The native resource token is invalid or expired"))]
  NativeResourceTokenInvalid,
  /// The signature verified but the credential is not an API access token.
  #[snafu(display("The presented token is not an access token for this API"))]
  ResourceTokenUse,

  #[snafu(display("Permission denied"))]
  PermissionError,
  #[snafu(display("Failed to find auth record"))]
  FindAuthRecordError,
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
}

impl AuthError {
  pub fn from_graphql_dynamic_subscribe_id_guard(source: async_graphql::Error, context: &ResolverContext, field_name: &str, column_name: &str) -> AuthError {
    AuthError::GraphqlDynamicPermissionError {
      source: Box::new(source),
      field: field_name.to_string(),
      column: column_name.to_string(),
      context_path: context.ctx.path_node.map(|p| p.to_string_vec().join("")).unwrap_or_default(),
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
      match &self {
        Self::Configuration { .. } | Self::IdentityStorage => StatusCode::INTERNAL_SERVER_ERROR,
        Self::PermissionError | Self::GraphqlDynamicPermissionError { .. } => StatusCode::FORBIDDEN,
        _ => StatusCode::UNAUTHORIZED,
      },
      Json(AuthErrorResponse::from(self)),
    )
      .into_response()
  }
}
