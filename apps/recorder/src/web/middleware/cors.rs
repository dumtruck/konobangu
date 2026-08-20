//! Configurable and Flexible CORS Middleware
//!
//! This middleware enables Cross-Origin Resource Sharing (CORS) by allowing
//! configurable origins, methods, and headers in HTTP requests. It can be
//! tailored to fit various application requirements, supporting permissive CORS
//! or specific rules as defined in the middleware configuration.

use std::{sync::Arc, time::Duration};

use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{self, Any};

use crate::{app::AppContextTrait, errors::RecorderResult, web::middleware::MiddlewareLayer};

/// Browser authentication defines the shared source and credential policy. A
/// permissive, independent CORS layer would invalidate that boundary.
pub fn for_auth(auth: &crate::auth::AuthService) -> RecorderResult<cors::CorsLayer> {
  let policy = auth.webui.clone();
  Ok(auth_layer(policy))
}

fn auth_layer(policy: crate::auth::WebuiPolicy) -> cors::CorsLayer {
  cors::CorsLayer::new()
    .allow_origin(cors::AllowOrigin::predicate(move |origin, _| {
      origin.to_str().is_ok_and(|origin| policy.allows_origin(origin))
    }))
    .allow_methods([http::Method::GET, http::Method::HEAD, http::Method::POST, http::Method::OPTIONS])
    .allow_headers([
      http::header::AUTHORIZATION,
      http::header::CONTENT_TYPE,
      http::header::RANGE,
      http::header::IF_RANGE,
      http::header::IF_NONE_MATCH,
      http::header::IF_MODIFIED_SINCE,
    ])
    .expose_headers([
      http::header::WWW_AUTHENTICATE,
      http::header::ETAG,
      http::header::LAST_MODIFIED,
      http::header::ACCEPT_RANGES,
      http::header::CONTENT_RANGE,
      http::header::CONTENT_LENGTH,
      http::header::CONTENT_ENCODING,
      http::header::VARY,
    ])
}

/// CORS middleware configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cors {
  #[serde(default)]
  pub enable: bool,
  /// Allow origins
  #[serde(default = "default_allow_origins")]
  pub allow_origins: Vec<String>,
  /// Allow headers
  #[serde(default = "default_allow_headers")]
  pub allow_headers: Vec<String>,
  /// Allow methods
  #[serde(default = "default_allow_methods")]
  pub allow_methods: Vec<String>,
  /// Allow credentials
  #[serde(default)]
  pub allow_credentials: bool,
  /// Max age
  pub max_age: Option<u64>,
  // Vary headers
  #[serde(default = "default_vary_headers")]
  pub vary: Vec<String>,
}

fn default_allow_origins() -> Vec<String> {
  vec!["*".to_string()]
}

fn default_allow_headers() -> Vec<String> {
  vec!["*".to_string()]
}

fn default_allow_methods() -> Vec<String> {
  vec!["*".to_string()]
}

fn default_vary_headers() -> Vec<String> {
  vec![
    "origin".to_string(),
    "access-control-request-method".to_string(),
    "access-control-request-headers".to_string(),
  ]
}

impl Default for Cors {
  fn default() -> Self {
    serde_json::from_value(json!({})).unwrap()
  }
}

impl Cors {
  /// Creates cors layer
  ///
  /// # Errors
  ///
  /// This function returns an error in the following cases:
  ///
  /// - If any of the provided origins in `allow_origins` cannot be parsed as a
  ///   valid URI, the function will return a parsing error.
  /// - If any of the provided headers in `allow_headers` cannot be parsed as
  ///   valid HTTP headers, the function will return a parsing error.
  /// - If any of the provided methods in `allow_methods` cannot be parsed as
  ///   valid HTTP methods, the function will return a parsing error.
  ///
  /// In all of these cases, the error returned will be the result of the
  /// `parse` method of the corresponding type.
  pub fn cors(&self) -> RecorderResult<cors::CorsLayer> {
    let mut cors: cors::CorsLayer = cors::CorsLayer::new();

    // testing CORS, assuming https://example.com in the allow list:
    // $ curl -v --request OPTIONS 'localhost:5150/api/_ping' -H 'Origin: https://example.com' -H 'Acces
    // look for '< access-control-allow-origin: https://example.com' in response.
    // if it doesn't appear (test with a bogus domain), it is not allowed.
    if self.allow_origins == default_allow_origins() {
      cors = cors.allow_origin(Any);
    } else {
      let mut list = vec![];
      for origin in &self.allow_origins {
        list.push(origin.parse()?);
      }
      if !list.is_empty() {
        cors = cors.allow_origin(list);
      }
    }

    if self.allow_headers == default_allow_headers() {
      cors = cors.allow_headers(Any);
    } else {
      let mut list = vec![];
      for header in &self.allow_headers {
        list.push(header.parse()?);
      }
      if !list.is_empty() {
        cors = cors.allow_headers(list);
      }
    }

    if self.allow_methods == default_allow_methods() {
      cors = cors.allow_methods(Any);
    } else {
      let mut list = vec![];
      for method in &self.allow_methods {
        list.push(method.parse()?);
      }
      if !list.is_empty() {
        cors = cors.allow_methods(list);
      }
    }

    let mut list = vec![];
    for v in &self.vary {
      list.push(v.parse()?);
    }
    if !list.is_empty() {
      cors = cors.vary(list);
    }

    if let Some(max_age) = self.max_age {
      cors = cors.max_age(Duration::from_secs(max_age));
    }

    cors = cors.allow_credentials(self.allow_credentials);

    Ok(cors)
  }
}

impl MiddlewareLayer for Cors {
  /// Returns the name of the middleware
  fn name(&self) -> &'static str {
    "cors"
  }

  /// Returns whether the middleware is enabled or not
  fn is_enabled(&self) -> bool {
    self.enable
  }

  fn config(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }

  /// Applies the CORS middleware layer to the Axum router.
  fn apply(&self, app: Router<Arc<dyn AppContextTrait>>) -> RecorderResult<Router<Arc<dyn AppContextTrait>>> {
    Ok(app.layer(self.cors()?))
  }
}

#[cfg(test)]
mod tests {
  use axum::{body::Body, routing::get};
  use tower::ServiceExt;

  use super::*;

  #[tokio::test]
  async fn authentication_cors_uses_allowed_origins_without_ambient_credentials() {
    let config: crate::auth::AuthConfig = serde_json::from_value(serde_json::json!({
      "webui":{"allowed_origins":[{"type":"strict","value":"https://ui.konobangu.test"},{"type":"strict","value":"http://192.168.1.20:5000"}]},
      "provider": {"type":"oidc","issuer":"https://issuer.test","browser":{"client_id":"public"}}
    }))
    .unwrap();
    let app = Router::new()
      .route("/private", get(|| async { http::StatusCode::UNAUTHORIZED }))
      .layer(auth_layer(config.webui_policy().unwrap()));
    for origin in ["https://ui.konobangu.test", "http://192.168.1.20:5000", "https://attacker.example"] {
      let preflight = app
        .clone()
        .oneshot(
          http::Request::builder()
            .method("OPTIONS")
            .uri("/private")
            .header("Origin", origin)
            .header("Access-Control-Request-Method", "GET")
            .header("Access-Control-Request-Headers", "authorization,range,if-none-match")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(preflight.status(), http::StatusCode::OK);
      if origin != "https://attacker.example" {
        assert_eq!(preflight.headers()["access-control-allow-origin"], origin);
      } else {
        assert!(preflight.headers().get("access-control-allow-origin").is_none());
      }
      assert!(preflight.headers().get("access-control-allow-credentials").is_none());
    }
    let denied = app
      .oneshot(
        http::Request::builder()
          .uri("/private")
          .header("Origin", "https://ui.konobangu.test")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(denied.status(), http::StatusCode::UNAUTHORIZED);
    assert_eq!(denied.headers()["access-control-allow-origin"], "https://ui.konobangu.test");
    assert!(denied.headers()["access-control-expose-headers"].to_str().unwrap().contains("content-range"));
  }
}
