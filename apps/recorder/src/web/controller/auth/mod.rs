use std::{
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use axum::{
  Extension, Json, Router,
  extract::{Request, State},
  http::header,
  middleware::{Next, from_fn, from_fn_with_state},
  response::Response,
  routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
  app::AppContextTrait,
  auth::{AuthError, AuthUserInfo, PublicAuthConfig, auth_middleware},
  errors::RecorderResult,
  web::controller::core::Controller,
};

pub const CONTROLLER_PREFIX: &str = "/api/auth";

/// The application principal. This is the user-facing projection, kept separate
/// from the verified bearer-token facts a resource server sees. The browser
/// derives its identity epoch from these fields, so the issuer comes from this
/// deployment's configuration rather than from any browser claim.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationPrincipal {
  pub subject: String,
  pub display_name: String,
  pub auth_type: String,
  pub issuer: String,
}

/// Public and unauthenticated by contract: the browser must read its login mode
/// before any auth client exists. It creates no session, sets no cookie and
/// does not query the identity store.
async fn config(State(ctx): State<Arc<dyn AppContextTrait>>, headers: http::HeaderMap) -> Result<Json<PublicAuthConfig>, http::StatusCode> {
  let generated_at_ms = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
    .unwrap_or_default();
  ctx.auth().public_config(&headers, generated_at_ms).map(Json)
}

/// Authenticated in both modes: the server verifies the presented credential
/// and projects the application identity it mapped.
async fn user_info(Extension(user): Extension<AuthUserInfo>) -> Result<Json<ApplicationPrincipal>, AuthError> {
  Ok(Json(ApplicationPrincipal {
    issuer: user.subscriber_auth.issuer.clone().ok_or(AuthError::Unauthorized)?,
    subject: user.subscriber_auth.pid.clone(),
    display_name: if user.is_admin {
      "Administrator".into()
    } else {
      user.subscriber_auth.pid.clone()
    },
    auth_type: "oidc".into(),
  }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceGrantRequest {}

async fn resource_grants(
  State(ctx): State<Arc<dyn AppContextTrait>>,
  Extension(user): Extension<AuthUserInfo>,
  Json(_request): Json<ResourceGrantRequest>,
) -> Result<Json<crate::auth::native_resources::NativeResourceGrant>, AuthError> {
  ctx.auth().resource_grants.issue(user.subscriber_auth.subscriber_id).map(Json)
}

async fn no_store(request: Request, next: Next) -> Response {
  let mut response = next.run(request).await;
  response
    .headers_mut()
    .insert(header::CACHE_CONTROL, header::HeaderValue::from_static("no-store"));
  response.headers_mut().append(header::VARY, header::HeaderValue::from_static("Origin, Referer"));
  response
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
  let router = Router::<Arc<dyn AppContextTrait>>::new()
    .route("/config", get(config))
    .route(
      "/resource-grants",
      post(resource_grants).layer(from_fn_with_state(ctx.clone(), auth_middleware)),
    )
    .route("/user-info", get(user_info).layer(from_fn_with_state(ctx.clone(), auth_middleware)))
    .layer(from_fn(no_store));
  Ok(Controller::from_nest_router(CONTROLLER_PREFIX, router))
}
