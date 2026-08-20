use std::{net::SocketAddr, sync::Arc};

use axum::{
  Json, Router,
  extract::{ConnectInfo, Form, Query, State},
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::{IntoResponse, Response},
  routing::{get, post},
};
use chrono::Utc;
use oxide_auth_async::code_grant;
use securitydept_core::{basic_auth_context::BasicAuthContextService, realip::TransportContext};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

use crate::{
  ProviderRuntime,
  engine::{AuthorizationRequest, Flow, TokenRequest},
};

pub(crate) fn router(runtime: Arc<ProviderRuntime>) -> Router {
  let origins: Vec<HeaderValue> = runtime.config.client_origins().iter().filter_map(|value| value.parse().ok()).collect();
  Router::new()
    .route("/.well-known/openid-configuration", get(metadata))
    .route("/jwks", get(jwks))
    .route("/authorize", get(authorize))
    .route("/login", get(login))
    .route("/token", post(token))
    .route("/userinfo", get(userinfo))
    .layer(
      CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
    )
    .layer(axum::middleware::map_response(secure_protocol_errors))
    .with_state(runtime)
}
async fn secure_protocol_errors(response: Response) -> Response {
  if response.status().is_success() { response } else { secured(response) }
}
#[derive(Debug)]
enum BasicWorkError {
  Busy,
  Failed,
}
async fn basic_work<T: Send + 'static>(budget: Arc<tokio::sync::Semaphore>, work: impl FnOnce() -> T + Send + 'static) -> Result<T, BasicWorkError> {
  let permit = budget.try_acquire_owned().map_err(|_| BasicWorkError::Busy)?;
  tokio::task::spawn_blocking(move || {
    let _permit = permit;
    work()
  })
  .await
  .map_err(|_| BasicWorkError::Failed)
}
fn secured(mut response: Response) -> Response {
  response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
  response.headers_mut().insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
  response.headers_mut().insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
  response
}
fn error(code: &'static str, status: StatusCode) -> Response {
  secured((status, Json(serde_json::json!({"error":code}))).into_response())
}
fn redirect(location: &str) -> Response {
  match HeaderValue::from_str(location) {
    Ok(value) => {
      let mut response = StatusCode::FOUND.into_response();
      response.headers_mut().insert(header::LOCATION, value);
      secured(response)
    }
    Err(_) => error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  }
}
async fn metadata(State(runtime): State<Arc<ProviderRuntime>>) -> Response {
  Json(runtime.metadata()).into_response()
}
async fn jwks(State(runtime): State<Arc<ProviderRuntime>>) -> Response {
  Json(runtime.jwks()).into_response()
}
async fn authorize(
  State(runtime): State<Arc<ProviderRuntime>>,
  query: Result<Query<AuthorizationRequest>, axum::extract::rejection::QueryRejection>,
) -> Response {
  let Ok(Query(request)) = query else {
    return error("invalid_request", StatusCode::BAD_REQUEST);
  };
  let mut flow = Flow::new(runtime.clone());
  // No user challenge or client redirect occurs before the engine validates it.
  if code_grant::authorization::authorization_code(&mut flow, &request).await.is_err() {
    return error("invalid_request", StatusCode::BAD_REQUEST);
  }
  let expires = Utc::now().timestamp() + 300;
  let pending = match runtime.store.put("pending", &request, expires).await {
    Ok(value) => value,
    Err(_) => return error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  };
  let mut login = url::Url::parse(&runtime.config.endpoint("login")).expect("validated issuer");
  login.query_pairs_mut().append_pair("pending", &pending);
  redirect(login.as_str())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginRequest {
  pending: String,
}
#[axum::debug_handler]
async fn login(
  State(runtime): State<Arc<ProviderRuntime>>,
  peer: Result<ConnectInfo<SocketAddr>, axum::extract::rejection::ExtensionRejection>,
  headers: HeaderMap,
  query: Result<Query<LoginRequest>, axum::extract::rejection::QueryRejection>,
) -> Response {
  let Ok(Query(request)) = query else {
    return error("invalid_request", StatusCode::BAD_REQUEST);
  };
  match runtime.store.exists(&request.pending, "pending").await {
    Ok(true) => {}
    Ok(false) => return error("invalid_request", StatusCode::BAD_REQUEST),
    Err(_) => return error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  };
  let Ok(ConnectInfo(peer)) = peer else {
    return error("server_error", StatusCode::INTERNAL_SERVER_ERROR);
  };
  let resolved = runtime.real_ip.resolve(peer.ip(), &headers, &TransportContext::default()).await;
  let authorization = match headers.get(header::AUTHORIZATION) {
    Some(value) => match value.to_str() {
      Ok(value) => Some(value.to_owned()),
      Err(_) => return error("invalid_request", StatusCode::BAD_REQUEST),
    },
    None => None,
  };
  if headers.get_all(header::AUTHORIZATION).iter().count() > 1 {
    return error("invalid_request", StatusCode::BAD_REQUEST);
  }
  let context = runtime.basic.clone();
  // SDK Argon2 verification is CPU work; it does not occupy an async worker.
  let verified = basic_work(runtime.basic_budget.clone(), move || {
    BasicAuthContextService::new(&context).and_then(|service| service.login("/auth-provider/login", authorization.as_deref(), None, Some(&resolved)))
  })
  .await;
  let response = match verified {
    Ok(Ok(response)) => response,
    Err(BasicWorkError::Busy) => return error("temporarily_unavailable", StatusCode::SERVICE_UNAVAILABLE),
    _ => return error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  };
  if response.status != StatusCode::FOUND {
    let mut result = response.status.into_response();
    *result.headers_mut() = response.headers;
    return secured(result);
  }
  let request = match runtime.store.take::<AuthorizationRequest>(&request.pending, "pending").await {
    Ok(Some(value)) => value,
    Ok(None) => return error("invalid_request", StatusCode::BAD_REQUEST),
    Err(_) => return error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  };
  let mut flow = Flow::new(runtime.clone());
  flow.bind_redirect(request.redirect_uri.clone());
  let pending = match code_grant::authorization::authorization_code(&mut flow, &request).await {
    Ok(value) => value,
    Err(_) => return error("invalid_request", StatusCode::BAD_REQUEST),
  };
  match pending
    .authorize(&mut flow, std::borrow::Cow::Borrowed(&runtime.config.credentials.subject))
    .await
  {
    Ok(location) => redirect(location.as_str()),
    Err(_) => error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  }
}
fn protocol_error(json: String) -> Response {
  secured((StatusCode::BAD_REQUEST, [(header::CONTENT_TYPE, "application/json")], json).into_response())
}
async fn token(
  State(runtime): State<Arc<ProviderRuntime>>,
  headers: HeaderMap,
  form: Result<Form<TokenRequest>, axum::extract::rejection::FormRejection>,
) -> Response {
  if headers.contains_key(header::AUTHORIZATION) {
    return error("invalid_client", StatusCode::BAD_REQUEST);
  }
  let Ok(Form(request)) = form else {
    return error("invalid_request", StatusCode::BAD_REQUEST);
  };
  if request.grant_type.is_none() || request.client_id.is_none() {
    return error("invalid_request", StatusCode::BAD_REQUEST);
  }
  if request.client_id.as_deref() != Some(&runtime.config.client.client_id) {
    return error("invalid_client", StatusCode::BAD_REQUEST);
  }
  let mut flow = Flow::new(runtime);
  flow.bind_redirect(request.redirect_uri.clone());
  use oxide_auth::code_grant::{accesstoken, error::AccessTokenErrorType, refresh};
  match request.grant_type.as_deref() {
    Some("authorization_code") => {
      if !request.well_formed()
        || request.code.as_ref().is_none_or(String::is_empty)
        || request.redirect_uri.as_ref().is_none_or(String::is_empty)
        || request.code_verifier.is_none()
      {
        return error("invalid_request", StatusCode::BAD_REQUEST);
      }
      match code_grant::access_token::access_token(&mut flow, &request).await {
        Ok(_) => {}
        Err(accesstoken::Error::Primitive(_)) => return error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
        Err(accesstoken::Error::Invalid(mut description)) => {
          // The engine's generic invalid-request error after recovering a code
          // means its binding or PKCE failed, which OAuth defines as
          // invalid_grant.
          if flow.code_queried() && matches!(description.description().kind(), AccessTokenErrorType::InvalidRequest) {
            description.description().set_type(AccessTokenErrorType::InvalidGrant);
          }
          return protocol_error(description.to_json());
        }
        Err(accesstoken::Error::Unauthorized(description, _)) => return protocol_error(description.to_json()),
      }
    }
    Some("refresh_token") => {
      if request.refresh_token.as_ref().is_none_or(String::is_empty) {
        return error("invalid_request", StatusCode::BAD_REQUEST);
      }
      match code_grant::refresh::refresh(&mut flow, &request).await {
        Ok(_) => {}
        Err(refresh::Error::Primitive) if flow.invalid_grant() => return error("invalid_grant", StatusCode::BAD_REQUEST),
        Err(refresh::Error::Primitive) => return error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
        Err(refresh::Error::Invalid(description) | refresh::Error::Unauthorized(description, _)) => return protocol_error(description.to_json()),
      }
    }
    _ => return error("unsupported_grant_type", StatusCode::BAD_REQUEST),
  }
  match flow.take_response() {
    Some(response) => secured(Json(response).into_response()),
    None => error("server_error", StatusCode::INTERNAL_SERVER_ERROR),
  }
}
async fn userinfo(State(runtime): State<Arc<ProviderRuntime>>, headers: HeaderMap) -> Response {
  let bearer = headers
    .get(header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.split_once(' '))
    .filter(|(scheme, token)| scheme.eq_ignore_ascii_case("Bearer") && !token.is_empty())
    .map(|(_, token)| token);
  let principal = match bearer {
    Some(token) => runtime.signer.verify_userinfo(&runtime.config, token).ok(),
    None => None,
  };
  match principal {
    Some(principal) => secured(Json(serde_json::json!({"sub":principal.subject})).into_response()),
    None => {
      let mut response = error("invalid_token", StatusCode::UNAUTHORIZED);
      response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer error=\"invalid_token\""));
      response
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn cancelled_request_retains_cpu_admission_until_blocking_work_finishes() {
    let budget = Arc::new(tokio::sync::Semaphore::new(2));
    let (started1, ready1) = tokio::sync::oneshot::channel();
    let (started2, ready2) = tokio::sync::oneshot::channel();
    let (release1, blocked1) = std::sync::mpsc::channel();
    let (release2, blocked2) = std::sync::mpsc::channel();
    let task1 = tokio::spawn(basic_work(budget.clone(), move || {
      started1.send(()).unwrap();
      blocked1.recv().unwrap();
    }));
    let task2 = tokio::spawn(basic_work(budget.clone(), move || {
      started2.send(()).unwrap();
      blocked2.recv().unwrap();
    }));
    ready1.await.unwrap();
    ready2.await.unwrap();
    assert!(matches!(basic_work(budget.clone(), || ()).await, Err(BasicWorkError::Busy)));
    task1.abort();
    assert!(task1.await.unwrap_err().is_cancelled());
    assert!(matches!(basic_work(budget.clone(), || ()).await, Err(BasicWorkError::Busy)));
    release1.send(()).unwrap();
    release2.send(()).unwrap();
    task2.await.unwrap().unwrap();
    let recovered = budget.clone().acquire_many_owned(2).await.unwrap();
    drop(recovered);
    assert!(basic_work(budget, || ()).await.is_ok());
  }
}
