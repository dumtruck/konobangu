//! Authentication configuration, grouped by responsibility.
//!
//! The WebUI always uses OIDC; provider selection only chooses the issuer
//! source.
use securitydept_utils::redirect::RedirectTargetRule;
use serde::{Deserialize, Serialize};

use super::webui::WebuiPolicy;

/// Default API audience and scope. These are deliberately distinct from the
/// browser client ID: an access token for this API is not an ID token.
pub const DEFAULT_RESOURCE_AUDIENCE: &str = "urn:konobangu:api";
pub const DEFAULT_RESOURCE_SCOPE: &str = "konobangu:api";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebuiAuthConfig {
  /// Browser entry rules validated into one immutable policy at startup.
  #[serde(with = "origin_rules")]
  pub allowed_origins: Vec<RedirectTargetRule>,
}

/// Keep the operator-facing wire format flat while the SDK owns matching.
mod origin_rules {
  use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::Error};

  use super::RedirectTargetRule;

  #[derive(Serialize, Deserialize)]
  #[serde(tag = "type", content = "value", rename_all = "snake_case", deny_unknown_fields)]
  enum Rule {
    Strict(String),
    Regex(String),
  }

  pub fn serialize<S: Serializer>(rules: &[RedirectTargetRule], serializer: S) -> Result<S::Ok, S::Error> {
    let rules = rules
      .iter()
      .map(|rule| match rule {
        RedirectTargetRule::Strict { value } => Ok(Rule::Strict(value.clone())),
        RedirectTargetRule::Regex { value } => Ok(Rule::Regex(value.as_str().to_owned())),
        RedirectTargetRule::All => Err(S::Error::custom("all WebUI origins are not allowed")),
      })
      .collect::<Result<Vec<_>, _>>()?;
    rules.serialize(serializer)
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<RedirectTargetRule>, D::Error> {
    Vec::<Rule>::deserialize(deserializer)?
      .into_iter()
      .map(|rule| match rule {
        Rule::Strict(value) => Ok(RedirectTargetRule::Strict { value }),
        Rule::Regex(value) => regex::Regex::new(&value)
          .map(|value| RedirectTargetRule::Regex { value })
          .map_err(serde::de::Error::custom),
      })
      .collect()
  }
}

#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityAuthConfig {
  /// Inherits `database.url` when omitted.
  #[serde(default)]
  pub database_url: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OidcBrowserConfig {
  /// The public client the browser uses for Authorization Code with PKCE.
  pub client_id: String,
  #[serde(default = "default_browser_scopes")]
  pub scopes: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OidcResourceConfig {
  #[serde(default = "default_audience")]
  pub audience: String,
  #[serde(default = "default_required_scopes")]
  pub required_scopes: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OidcAuthConfig {
  pub issuer: String,
  /// PEM certificate authorities trusted for the issuer in addition to the
  /// platform roots, for an identity provider behind a private CA.
  #[serde(default)]
  pub trusted_ca_file: Option<std::path::PathBuf>,
  pub browser: OidcBrowserConfig,
  #[serde(default)]
  pub resource: OidcResourceConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProviderAuthConfig {
  Bundled(Box<konobangu_auth_provider::ProviderConfig>),
  Oidc(OidcAuthConfig),
}

/// Administrator grants are matched against verified issuer and subject only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationAdministrator {
  pub issuer: String,
  pub subject: String,
}

/// Independent signing material for native resource capabilities.
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceGrantsConfig {
  #[serde(default)]
  pub signing_key: Option<String>,
}

impl std::fmt::Debug for ResourceGrantsConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ResourceGrantsConfig")
      .field("signing_key", &self.signing_key.as_ref().map(|_| "[REDACTED]"))
      .finish()
  }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
  #[serde(default)]
  pub resource_grants: ResourceGrantsConfig,
  pub webui: WebuiAuthConfig,
  #[serde(default)]
  pub identity: IdentityAuthConfig,
  pub provider: ProviderAuthConfig,
  #[serde(default)]
  pub administrators: Vec<ApplicationAdministrator>,
}

fn default_browser_scopes() -> Vec<String> {
  vec!["openid".to_owned(), "profile".to_owned(), DEFAULT_RESOURCE_SCOPE.to_owned()]
}
fn default_audience() -> String {
  DEFAULT_RESOURCE_AUDIENCE.to_owned()
}
fn default_required_scopes() -> Vec<String> {
  vec![DEFAULT_RESOURCE_SCOPE.to_owned()]
}

impl Default for OidcResourceConfig {
  fn default() -> Self {
    Self {
      audience: default_audience(),
      required_scopes: default_required_scopes(),
    }
  }
}

impl AuthConfig {
  pub fn webui_policy(&self) -> Result<WebuiPolicy, super::AuthError> {
    WebuiPolicy::new(&self.webui.allowed_origins).map_err(|error| super::AuthError::Configuration { message: error.to_string() })
  }

  pub fn oidc(&self) -> Option<&OidcAuthConfig> {
    match &self.provider {
      ProviderAuthConfig::Oidc(config) => Some(config),
      ProviderAuthConfig::Bundled(_) => None,
    }
  }
}

// Secrets are redacted in every debug form; configuration errors must never
// print a password or a connection string.
impl std::fmt::Debug for IdentityAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("IdentityAuthConfig").field("database_url", &"[REDACTED]").finish()
  }
}

impl std::fmt::Debug for WebuiAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WebuiAuthConfig").field("allowed_origins", &self.allowed_origins).finish()
  }
}

impl std::fmt::Debug for OidcBrowserConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OidcBrowserConfig")
      .field("client_id", &self.client_id)
      .field("scopes", &self.scopes)
      .finish()
  }
}

impl std::fmt::Debug for OidcResourceConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OidcResourceConfig")
      .field("audience", &self.audience)
      .field("required_scopes", &self.required_scopes)
      .finish()
  }
}

impl std::fmt::Debug for OidcAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OidcAuthConfig")
      .field("issuer", &self.issuer)
      .field("trusted_ca_file", &self.trusted_ca_file)
      .field("browser", &self.browser)
      .field("resource", &self.resource)
      .finish()
  }
}

impl std::fmt::Debug for ProviderAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Bundled(config) => f.debug_tuple("Bundled").field(config).finish(),
      Self::Oidc(config) => f.debug_tuple("Oidc").field(config).finish(),
    }
  }
}

impl std::fmt::Debug for AuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AuthConfig")
      .field("webui", &self.webui)
      .field("identity", &self.identity)
      .field("resource_grants", &self.resource_grants)
      .field("administrators", &self.administrators)
      .field("provider", &self.provider)
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn origin_rules_use_a_flat_wire_format_and_sdk_matching() {
    let webui: WebuiAuthConfig = serde_json::from_value(serde_json::json!({
      "allowed_origins": [
        {"type":"strict","value":"http://192.168.1.20:5000/"},
        {"type":"regex","value":"https://app-[a-z]+\\.region\\.example"}
      ]
    }))
    .unwrap();
    let policy = WebuiPolicy::new(&webui.allowed_origins).unwrap();
    assert!(policy.allows_origin("http://192.168.1.20:5000"));
    assert!(policy.allows_origin("https://app-east.region.example"));
    let wire = serde_json::to_value(&webui).unwrap();
    assert_eq!(wire["allowed_origins"][0]["value"], "http://192.168.1.20:5000/");
    assert!(serde_json::from_value::<WebuiAuthConfig>(serde_json::json!({"allowed_origins":[{"type":"all"}]})).is_err());
    assert!(serde_json::from_value::<WebuiAuthConfig>(serde_json::json!({"public_url":"https://ui.example"})).is_err());
  }

  #[test]
  fn resource_defaults_separate_the_api_audience_from_the_browser_client() {
    let config: AuthConfig = serde_json::from_value(serde_json::json!({
      "webui": { "allowed_origins": [{ "type": "strict", "value": "https://ui.konobangu.test/" }] },
      "provider": { "type": "oidc", "issuer": "https://idp.example/", "browser": { "client_id": "konobangu-webui" } }
    }))
    .unwrap();
    let oidc = config.oidc().unwrap();
    assert_eq!(oidc.resource.audience, DEFAULT_RESOURCE_AUDIENCE);
    assert_eq!(oidc.resource.required_scopes, vec![DEFAULT_RESOURCE_SCOPE.to_owned()]);
    assert_ne!(oidc.resource.audience, oidc.browser.client_id);
    assert!(oidc.browser.scopes.contains(&DEFAULT_RESOURCE_SCOPE.to_owned()));
    assert!(oidc.browser.scopes.contains(&"openid".to_owned()));
  }

  #[test]
  fn rejects_retired_and_unknown_fields() {
    for provider in [
      // A frontend-oidc public client has no secret, and executable claims
      // scripts are never configured here.
      serde_json::json!({ "type": "oidc", "issuer": "https://idp.example/", "client_secret": "s",
                          "browser": { "client_id": "c" } }),
      serde_json::json!({ "type": "oidc", "issuer": "https://idp.example/", "extra_claims": {},
                          "browser": { "client_id": "c" } }),
      serde_json::json!({ "type": "oidc", "issuer": "https://idp.example/",
                          "browser": { "client_id": "c", "claims_check_script": "x" } }),
      serde_json::json!({ "type": "basic", "username": "u", "password": "p", "session": {} }),
    ] {
      let parsed = serde_json::from_value::<AuthConfig>(serde_json::json!({
        "webui": { "allowed_origins": [{ "type": "strict", "value": "https://ui.konobangu.test/" }] },
        "provider": provider
      }));
      assert!(parsed.is_err(), "{provider} must be rejected");
    }
  }

  #[test]
  fn debug_forms_redact_every_secret() {
    let config: AuthConfig = serde_json::from_value(serde_json::json!({
      "webui": { "allowed_origins": [{ "type": "strict", "value": "https://ui.konobangu.test/" }] },
      "identity": { "database_url": "postgres://user:hunter2@localhost/konobangu" },
      "provider": { "type": "bundled", "issuer": "https://issuer.test/auth-provider", "client": { "client_id": "c", "redirect_uris": ["https://ui.konobangu.test/auth/callback"] }, "credentials": { "username": "admin", "password": "hunter2" }, "signing": { "private_key_pem": "hunter2" } }
    }))
    .unwrap();
    let rendered = format!("{config:?}");
    assert!(!rendered.contains("hunter2"), "{rendered}");
    assert!(rendered.contains("[REDACTED]"));
  }
}
