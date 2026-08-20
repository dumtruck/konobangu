//! Sets secure headers for your backend to promote security-by-default.
//!
//! This middleware applies secure HTTP headers, providing pre-defined presets
//! (e.g., "github") and the ability to override or define custom headers.

use std::{
  collections::{BTreeMap, HashMap},
  sync::{Arc, OnceLock},
  task::{Context, Poll},
};

use axum::{
  Router,
  body::Body,
  http::{HeaderMap, HeaderName, HeaderValue, Request},
  response::Response,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use snafu::whatever;
use tower::{Layer, Service};

use crate::{app::AppContextTrait, errors::RecorderResult, web::middleware::MiddlewareLayer};

static PRESETS: OnceLock<HashMap<String, BTreeMap<String, String>>> = OnceLock::new();
fn get_presets() -> &'static HashMap<String, BTreeMap<String, String>> {
  PRESETS.get_or_init(|| {
    let json_data = include_str!("secure_headers.json");
    serde_json::from_str(json_data).unwrap()
  })
}
/// Response defaults from a preset and optional configured overrides.
///
/// Overrides replace preset values. Explicit endpoint headers take precedence
/// over both, so scoped CSP and cache policies survive generic hardening.
/// `empty` starts without preset defaults. Imperative `Clear-Site-Data` belongs
/// to a deliberate operation or override, not every response.
///
/// ```toml
/// [server.middlewares.secure_headers]
/// enable = true
/// preset = "github"
/// [server.middlewares.secure_headers.overrides]
/// "Referrer-Policy" = "same-origin"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecureHeader {
  #[serde(default)]
  pub enable: bool,
  #[serde(default = "default_preset")]
  pub preset: String,
  #[serde(default)]
  pub overrides: Option<BTreeMap<String, String>>,
}

impl Default for SecureHeader {
  fn default() -> Self {
    serde_json::from_value(json!({})).unwrap()
  }
}

fn default_preset() -> String {
  "github".to_string()
}

impl MiddlewareLayer for SecureHeader {
  /// Returns the name of the middleware
  fn name(&self) -> &'static str {
    "secure_headers"
  }

  /// Returns whether the middleware is enabled or not
  fn is_enabled(&self) -> bool {
    self.enable
  }

  fn config(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }

  /// Applies the secure headers layer to the application router
  fn apply(&self, app: Router<Arc<dyn AppContextTrait>>) -> RecorderResult<Router<Arc<dyn AppContextTrait>>> {
    Ok(app.layer(SecureHeaders::new(self)?))
  }
}

impl SecureHeader {
  /// Converts the configuration into a single map of response defaults.
  ///
  /// Applies the preset headers and any custom overrides.
  fn as_headers(&self) -> RecorderResult<HeaderMap> {
    let mut headers = HeaderMap::new();

    let preset = &self.preset;
    if let Some(p) = get_presets().get(preset) {
      Self::merge_headers(&mut headers, p)?;
      if let Some(overrides) = &self.overrides {
        Self::merge_headers(&mut headers, overrides)?;
      }
      Ok(headers)
    } else {
      whatever!("secure_headers: a preset named `{preset}` does not exist")
    }
  }

  /// Normalizing names before merging lets configured overrides win regardless
  /// of casing, without retaining two defaults for the same response header.
  fn merge_headers(headers: &mut HeaderMap, hm: &BTreeMap<String, String>) -> RecorderResult<()> {
    for (k, v) in hm {
      headers.insert(HeaderName::from_bytes(k.as_bytes())?, HeaderValue::from_str(v)?);
    }
    Ok(())
  }
}

/// The [`SecureHeaders`] layer which wraps around the service and injects
/// security headers
#[derive(Clone, Debug)]
pub struct SecureHeaders {
  headers: HeaderMap,
}

impl SecureHeaders {
  /// Creates a new [`SecureHeaders`] instance with the provided
  /// configuration.
  ///
  /// # Errors
  /// Returns an error if any header values are invalid.
  pub fn new(config: &SecureHeader) -> RecorderResult<Self> {
    Ok(Self { headers: config.as_headers()? })
  }
}

impl<S> Layer<S> for SecureHeaders {
  type Service = SecureHeadersMiddleware<S>;

  /// Wraps the provided service with the secure headers middleware.
  fn layer(&self, inner: S) -> Self::Service {
    SecureHeadersMiddleware { inner, layer: self.clone() }
  }
}

/// The secure headers middleware
#[derive(Clone, Debug)]
#[must_use]
pub struct SecureHeadersMiddleware<S> {
  inner: S,
  layer: SecureHeaders,
}

impl<S> Service<Request<Body>> for SecureHeadersMiddleware<S>
where
  S: Service<Request<Body>, Response = Response> + Send + 'static,
  S::Future: Send + 'static,
{
  type Response = S::Response;
  type Error = S::Error;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, request: Request<Body>) -> Self::Future {
    let layer = self.layer.clone();
    let future = self.inner.call(request);
    Box::pin(async move {
      let mut response: Response = future.await?;
      let headers = response.headers_mut();
      for (k, v) in &layer.headers {
        // Endpoints own scoped CSP, cache and referrer policies. This layer
        // supplies response defaults, just like
        // SetResponseHeaderLayer::if_not_present.
        headers.entry(k.clone()).or_insert(v.clone());
      }
      Ok(response)
    })
  }
}

#[cfg(test)]
mod tests {

  use axum::{
    Router,
    http::{HeaderMap, Method},
    routing::get,
  };
  use insta::assert_debug_snapshot;
  use tower::ServiceExt;

  use super::*;
  fn normalize_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
      .iter()
      .map(|(k, v)| {
        let key = k.to_string();
        let value = v.to_str().unwrap_or("").to_string();
        (key, value)
      })
      .collect()
  }
  #[tokio::test]
  async fn can_set_headers() {
    let config = SecureHeader {
      enable: true,
      preset: "github".to_string(),
      overrides: None,
    };
    let app = Router::new().route("/", get(|| async {})).layer(SecureHeaders::new(&config).unwrap());

    let req = Request::builder().uri("/").method(Method::GET).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_debug_snapshot!(normalize_headers(response.headers()));
  }

  #[tokio::test]
  async fn can_override_headers() {
    let mut overrides = BTreeMap::new();
    overrides.insert("X-Download-Options".to_string(), "foobar".to_string());
    overrides.insert("New-Header".to_string(), "baz".to_string());

    let config = SecureHeader {
      enable: true,
      preset: "github".to_string(),
      overrides: Some(overrides),
    };
    let app = Router::new().route("/", get(|| async {})).layer(SecureHeaders::new(&config).unwrap());

    let req = Request::builder().uri("/").method(Method::GET).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_debug_snapshot!(normalize_headers(response.headers()));
  }

  #[tokio::test]
  async fn default_is_github_preset() {
    let config = SecureHeader::default();
    let app = Router::new().route("/", get(|| async {})).layer(SecureHeaders::new(&config).unwrap());

    let req = Request::builder().uri("/").method(Method::GET).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_debug_snapshot!(normalize_headers(response.headers()));
  }

  #[tokio::test]
  async fn both_presets_preserve_endpoint_policies_and_keep_configured_defaults() {
    for preset in ["github", "owasp"] {
      let config = SecureHeader {
        enable: true,
        preset: preset.to_owned(),
        overrides: Some(BTreeMap::from([
          ("x-content-type-options".to_owned(), "configured-default".to_owned()),
          ("Content-Security-Policy".to_owned(), "default-src 'none'; frame-ancestors 'none'".to_owned()),
          ("Cache-Control".to_owned(), "public, max-age=100".to_owned()),
        ])),
      };
      let app = Router::new()
        .route(
          "/",
          get(|| async {
            (
              [
                ("content-security-policy", "default-src 'none'; frame-ancestors https://ui.example"),
                ("cache-control", "no-store"),
                ("referrer-policy", "no-referrer"),
              ],
              "Endpoint-owned document",
            )
          }),
        )
        .layer(SecureHeaders::new(&config).unwrap());
      let response = app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
      assert_eq!(
        response.headers()["content-security-policy"],
        "default-src 'none'; frame-ancestors https://ui.example"
      );
      assert_eq!(response.headers()["cache-control"], "no-store");
      assert_eq!(response.headers()["referrer-policy"], "no-referrer");
      assert_eq!(response.headers()["x-content-type-options"], "configured-default");
      assert!(response.headers().contains_key("strict-transport-security"));
      assert!(!response.headers().contains_key("clear-site-data"));
    }
  }

  #[tokio::test]
  async fn explicitly_configured_state_clearing_remains_available() {
    let config = SecureHeader {
      enable: true,
      preset: "owasp".to_owned(),
      overrides: Some(BTreeMap::from([("Clear-Site-Data".to_owned(), "\"cookies\"".to_owned())])),
    };
    let app = Router::new().route("/", get(|| async {})).layer(SecureHeaders::new(&config).unwrap());
    let response = app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.headers()["clear-site-data"], "\"cookies\"");
  }
}
