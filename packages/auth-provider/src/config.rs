use std::fmt;

use securitydept_core::realip::{RealIpAccessConfig, RealIpResolveConfig};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::ProviderError;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ProviderConfig {
  pub issuer: String,
  pub client: ClientConfig,
  pub resource: ResourceConfig,
  pub credentials: CredentialsConfig,
  pub signing: SigningConfig,
  pub database: DatabaseConfig,
  pub tokens: TokenConfig,
  pub real_ip: Option<RealIpConfig>,
}

impl fmt::Debug for ProviderConfig {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ProviderConfig")
      .field("issuer", &self.issuer)
      .field("client", &self.client)
      .finish_non_exhaustive()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClientConfig {
  pub client_id: String,
  pub redirect_uris: Vec<String>,
  pub scopes: Vec<String>,
}
impl Default for ClientConfig {
  fn default() -> Self {
    Self {
      client_id: "konobangu-webui".into(),
      redirect_uris: vec![],
      scopes: ["openid", "profile", "konobangu:api", "offline_access"].map(str::to_owned).to_vec(),
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResourceConfig {
  pub audience: String,
  pub required_scopes: Vec<String>,
}
impl Default for ResourceConfig {
  fn default() -> Self {
    Self {
      audience: "urn:konobangu:api".into(),
      required_scopes: vec!["konobangu:api".into()],
    }
  }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CredentialsConfig {
  pub username: String,
  pub password: String,
  pub subject: String,
}
impl Default for CredentialsConfig {
  fn default() -> Self {
    Self {
      username: "admin".into(),
      password: String::new(),
      subject: "local-admin".into(),
    }
  }
}
impl fmt::Debug for CredentialsConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CredentialsConfig").field("subject", &self.subject).finish_non_exhaustive()
  }
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct SigningConfig {
  pub private_key_pem: String,
}
impl fmt::Debug for SigningConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("SigningConfig { private_key_pem: [REDACTED] }")
  }
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DatabaseConfig {
  pub url: String,
}
impl fmt::Debug for DatabaseConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("DatabaseConfig { url: [REDACTED] }")
  }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TokenConfig {
  pub authorization_code_ttl_seconds: u64,
  pub access_token_ttl_seconds: u64,
  pub refresh_token_ttl_seconds: u64,
}
impl Default for TokenConfig {
  fn default() -> Self {
    Self {
      authorization_code_ttl_seconds: 60,
      access_token_ttl_seconds: 300,
      refresh_token_ttl_seconds: 2_592_000,
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RealIpConfig {
  pub resolve: RealIpResolveConfig,
  pub access: Option<RealIpAccessConfig>,
}

impl ProviderConfig {
  pub fn validate(&self) -> Result<(), ProviderError> {
    let invalid = || ProviderError::Configuration;
    let issuer = Url::parse(&self.issuer).map_err(|_| invalid())?;
    if issuer.as_str() != self.issuer
      || !secure_url(&issuer)
      || issuer.query().is_some()
      || issuer.fragment().is_some()
      || !issuer.username().is_empty()
      || issuer.password().is_some()
      || !issuer.path().ends_with("/auth-provider")
    {
      return Err(invalid());
    }
    if self.client.client_id.trim().is_empty()
      || self.client.redirect_uris.is_empty()
      || !self.client.scopes.iter().any(|scope| scope == "openid")
      || self.resource.audience.trim().is_empty()
      || self.resource.required_scopes.is_empty()
      || !self.resource.required_scopes.iter().all(|scope| self.client.scopes.contains(scope))
    {
      return Err(invalid());
    }
    if self.client.scopes.iter().chain(self.resource.required_scopes.iter()).any(|scope| {
      scope.is_empty()
        || !scope
          .bytes()
          .all(|byte| byte == 0x21 || (0x23..=0x5b).contains(&byte) || (0x5d..=0x7e).contains(&byte))
    }) {
      return Err(invalid());
    }
    for callback in &self.client.redirect_uris {
      let uri = Url::parse(callback).map_err(|_| invalid())?;
      if !matches!(uri.scheme(), "https" | "http")
        || uri.host_str().is_none()
        || uri.fragment().is_some()
        || !uri.username().is_empty()
        || uri.password().is_some()
      {
        return Err(invalid());
      }
    }
    if self.credentials.username.is_empty()
      || self.credentials.username.contains(':')
      || self.credentials.password.is_empty()
      || self.credentials.subject.trim().is_empty()
      || self.database.url.is_empty()
      || self.tokens.authorization_code_ttl_seconds == 0
      || self.tokens.authorization_code_ttl_seconds > 600
      || self.tokens.access_token_ttl_seconds == 0
      || self.tokens.access_token_ttl_seconds > 86400
      || self.tokens.refresh_token_ttl_seconds == 0
      || self.tokens.refresh_token_ttl_seconds > 31_536_000
    {
      return Err(invalid());
    }
    Ok(())
  }
  pub fn endpoint(&self, name: &str) -> String {
    format!("{}/{name}", self.issuer)
  }
  pub fn login_path(&self) -> String {
    Url::parse(&self.endpoint("login")).expect("validated issuer").path().to_owned()
  }
  pub fn client_origins(&self) -> Vec<String> {
    self
      .client
      .redirect_uris
      .iter()
      .filter_map(|uri| Url::parse(uri).ok())
      .map(|uri| uri.origin().ascii_serialization())
      .collect()
  }
}
fn secure_url(uri: &Url) -> bool {
  uri.scheme() == "https" || (uri.scheme() == "http" && matches!(uri.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
}

#[cfg(test)]
mod tests {
  use super::*;
  fn configured() -> ProviderConfig {
    ProviderConfig {
      issuer: "https://api.example/auth-provider".into(),
      client: ClientConfig {
        redirect_uris: vec!["http://192.168.1.20:5000/auth/callback".into()],
        ..Default::default()
      },
      credentials: CredentialsConfig {
        password: "private-test-password".into(),
        ..Default::default()
      },
      database: DatabaseConfig {
        url: "postgres://private-test-url".into(),
      },
      ..Default::default()
    }
  }
  #[test]
  fn issuer_is_canonical_https_while_registered_callbacks_can_use_lan_http() {
    let mut config = configured();
    assert!(config.validate().is_ok());
    for issuer in [
      "http://192.168.1.20/auth-provider",
      "https://API.example/auth-provider",
      "https://api.example/a/../auth-provider",
      "https://api.example/auth-provider/",
      "https://user:password@api.example/auth-provider",
    ] {
      config.issuer = issuer.into();
      assert!(config.validate().is_err());
    }
    config.issuer = "http://127.0.0.1/auth-provider".into();
    assert!(config.validate().is_ok());
    for callback in [
      "https://ui.example/auth/callback#fragment",
      "https://user:password@ui.example/auth/callback",
      "file:///auth/callback",
    ] {
      config.client.redirect_uris = vec![callback.into()];
      assert!(config.validate().is_err());
    }
  }
  #[test]
  fn unknown_fields_invalid_scope_and_secret_debug_are_rejected() {
    assert!(serde_json::from_value::<ProviderConfig>(serde_json::json!({"obsolete":true})).is_err());
    let mut config = configured();
    for scope in ["", "openid profile", "bad\\scope", "bad\"scope"] {
      config.client.scopes.push(scope.into());
      assert!(config.validate().is_err());
      config.client.scopes.pop();
    }
    let debug = format!("{config:?} {:?} {:?}", config.credentials, config.database);
    assert!(!debug.contains("private-test"));
  }
}
