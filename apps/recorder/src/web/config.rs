use serde::{Deserialize, Serialize};

use super::middleware::MiddlewareConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebServerConfig {
  /// The address on which the server should listen on for incoming
  /// connections.
  #[serde(default = "default_binding")]
  pub binding: String,
  /// The port on which the server should listen for incoming connections.
  #[serde(default = "default_port")]
  pub port: u16,
  /// The webserver host
  pub host: String,
  /// Public API base used only for RSS self-links. Never an authentication
  /// source.
  #[serde(default)]
  pub external_base_url: securitydept_utils::base_url::ExternalBaseUrl,
  /// Identify via the `Server` header
  pub ident: Option<String>,
  /// Middleware configurations for the server, including payload limits,
  /// logging, and error handling.
  #[serde(default)]
  pub middlewares: MiddlewareConfig,
}

pub fn default_binding() -> String {
  "127.0.0.1".to_string()
}

pub fn default_port() -> u16 {
  5_001
}

impl WebServerConfig {
  pub fn validate_external_base_url(&self) -> Result<(), &'static str> {
    let securitydept_utils::base_url::ExternalBaseUrl::Fixed(value) = &self.external_base_url else {
      return Ok(());
    };
    let url = url::Url::parse(value).map_err(|_| "must be a valid HTTP(S) API URL")?;
    let userinfo = value
      .split_once("://")
      .is_some_and(|(_, remainder)| remainder.split(['/', '?', '#']).next().is_some_and(|authority| authority.contains('@')));
    if !matches!(url.scheme(), "http" | "https")
      || url.host_str().is_none()
      || !url.username().is_empty()
      || url.password().is_some()
      || userinfo
      || url.query().is_some()
      || url.fragment().is_some()
      || value.chars().any(char::is_whitespace)
      || value.contains('\\')
    {
      return Err("must be an HTTP(S) API URL without user information, query or fragment");
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fixed_rss_api_base_is_independent_of_browser_entries_and_checked_at_startup() {
    let mut server: WebServerConfig = serde_json::from_value(serde_json::json!({"host":"127.0.0.1"})).unwrap();
    assert!(server.validate_external_base_url().is_ok());
    for valid in ["https://api.example", "http://192.168.0.15:5001", "https://api.example/base"] {
      server.external_base_url = securitydept_utils::base_url::ExternalBaseUrl::Fixed(valid.into());
      assert!(server.validate_external_base_url().is_ok(), "{valid}");
    }
    for invalid in [
      "ftp://api.example",
      "https://u:pw@api.example",
      "https://@api.example",
      "https://api.example?q=1",
      "https://api.example#x",
      "/api",
    ] {
      server.external_base_url = securitydept_utils::base_url::ExternalBaseUrl::Fixed(invalid.into());
      assert!(server.validate_external_base_url().is_err(), "{invalid}");
    }
  }
}
