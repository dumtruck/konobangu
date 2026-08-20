//! The public authentication configuration DTO.
//!
//! The browser must learn its login mode before any auth client exists, so this
//! is served unauthenticated. For OIDC the SDK's own projection type is
//! flattened at the response root, with only this project's `provider` and
//! `resource` envelope added, so the SDK's network source consumes the
//! projection directly instead of an envelope it does not recognize.
use securitydept_token_set_context::frontend_oidc_mode::FrontendOidcModeConfigProjection;
use serde::Serialize;

use super::{AuthError, config::OidcAuthConfig, webui::WebuiOrigin};

/// The application always exposes the OIDC client model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicAuthProvider {
  Oidc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicResourceConfig {
  pub audience: String,
  pub required_scopes: Vec<String>,
}

/// The OIDC response: the SDK projection flattened at the root, plus this
/// project's envelope.
#[derive(Debug, Clone, Serialize)]
pub struct PublicOidcAuthConfig {
  pub provider: PublicAuthProvider,
  pub resource: PublicResourceConfig,
  #[serde(flatten)]
  pub projection: FrontendOidcModeConfigProjection,
}

pub type PublicAuthConfig = PublicOidcAuthConfig;

/// Builds public OIDC connectivity once at startup. Callback and freshness
/// remain empty until a request selects a verified browser entry.
pub fn oidc_projection_template(config: &OidcAuthConfig) -> Result<FrontendOidcModeConfigProjection, AuthError> {
  let discovery_base = config.issuer.trim_end_matches('/');
  // Only public connectivity is projected. Endpoint overrides stay empty so the
  // browser derives them from discovery, and the unsafe fields stay absent.
  let projection = FrontendOidcModeConfigProjection {
    well_known_url: Some(format!("{discovery_base}/.well-known/openid-configuration")),
    issuer_url: Some(config.issuer.clone()),
    jwks_uri: None,
    metadata_refresh_interval: std::time::Duration::ZERO,
    jwks_refresh_interval: std::time::Duration::ZERO,
    authorization_endpoint: None,
    token_endpoint: None,
    userinfo_endpoint: None,
    revocation_endpoint: None,
    token_endpoint_auth_methods_supported: None,
    id_token_signing_alg_values_supported: None,
    userinfo_signing_alg_values_supported: None,
    client_id: config.browser.client_id.clone(),
    client_secret: None,
    scopes: config.browser.scopes.clone(),
    required_scopes: config.resource.required_scopes.clone(),
    redirect_url: String::new(),
    pkce_enabled: true,
    claims_check_script: None,
    generated_at: 0,
  };
  enforce_host_policy(&projection)?;
  Ok(projection)
}

/// The SDK schema permits unsafe fields, so a structurally valid projection is
/// not necessarily a safe one. These are this host's own rules.
pub fn enforce_host_policy(projection: &FrontendOidcModeConfigProjection) -> Result<(), AuthError> {
  let invalid = |message: &str| AuthError::Configuration { message: message.into() };
  if projection.client_secret.is_some() {
    return Err(invalid("a browser OIDC client must be public; refusing to project a client secret"));
  }
  if projection.claims_check_script.is_some() {
    return Err(invalid("refusing to project an executable claims-check script to the browser"));
  }
  if !projection.pkce_enabled {
    return Err(invalid("PKCE is required for the browser authorization code flow"));
  }
  if projection.client_id.trim().is_empty() {
    return Err(invalid("auth.provider.browser.client_id must not be empty"));
  }
  if projection.issuer_url.as_deref().is_none_or(|issuer| issuer.trim().is_empty()) {
    return Err(invalid("auth.provider.issuer must not be empty"));
  }
  Ok(())
}

/// Copies public startup connectivity and binds it to this verified entry.
pub fn stamped(template: &FrontendOidcModeConfigProjection, webui: &WebuiOrigin, generated_at_ms: u64) -> FrontendOidcModeConfigProjection {
  let mut projection = template.clone();
  projection.redirect_url = webui.callback_url();
  projection.generated_at = generated_at_ms;
  projection
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::auth::config::{OidcBrowserConfig, OidcResourceConfig};

  fn webui() -> WebuiOrigin {
    WebuiOrigin::parse("https://ui.konobangu.test/").unwrap()
  }

  fn config() -> OidcAuthConfig {
    OidcAuthConfig {
      issuer: "https://idp.example/".into(),
      trusted_ca_file: None,
      browser: OidcBrowserConfig {
        client_id: "konobangu-webui".into(),
        scopes: vec!["openid".into(), "profile".into(), "konobangu:api".into()],
      },
      resource: OidcResourceConfig::default(),
    }
  }

  #[test]
  fn projects_public_connectivity_and_independent_request_callbacks() {
    let template = oidc_projection_template(&config()).unwrap();
    assert_eq!(template.client_id, "konobangu-webui");
    assert_eq!(template.issuer_url.as_deref(), Some("https://idp.example/"));
    assert_eq!(template.well_known_url.as_deref(), Some("https://idp.example/.well-known/openid-configuration"));
    assert!(template.redirect_url.is_empty());
    let selected = stamped(&template, &webui(), 17);
    assert_eq!(selected.redirect_url, "https://ui.konobangu.test/auth/callback");
    let other = WebuiOrigin::parse("http://192.168.0.50:5000").unwrap();
    assert_eq!(stamped(&template, &other, 18).redirect_url, "http://192.168.0.50:5000/auth/callback");
    assert!(template.redirect_url.is_empty());
    assert!(template.pkce_enabled);
    assert!(template.client_secret.is_none());
    assert!(template.claims_check_script.is_none());
  }

  #[test]
  fn host_policy_rejects_unsafe_projections_the_sdk_schema_would_accept() {
    let base = oidc_projection_template(&config()).unwrap();
    let mut secret = base.clone();
    secret.client_secret = Some("s".into());
    assert!(enforce_host_policy(&secret).is_err());
    let mut no_pkce = base.clone();
    no_pkce.pkce_enabled = false;
    assert!(enforce_host_policy(&no_pkce).is_err());
    let mut empty_client = base.clone();
    empty_client.client_id = "  ".into();
    assert!(enforce_host_policy(&empty_client).is_err());
    let mut empty_issuer = base;
    empty_issuer.issuer_url = Some(" ".into());
    assert!(enforce_host_policy(&empty_issuer).is_err());
  }

  #[test]
  fn oidc_response_flattens_the_projection_beside_the_envelope() {
    let template = oidc_projection_template(&config()).unwrap();
    let response = PublicAuthConfig {
      provider: PublicAuthProvider::Oidc,
      resource: PublicResourceConfig {
        audience: config().resource.audience,
        required_scopes: config().resource.required_scopes,
      },
      projection: stamped(&template, &webui(), 1_700_000_000_000),
    };
    let json = serde_json::to_value(&response).unwrap();
    // The SDK projection serializes camelCase and the network source reads it
    // from the root object, so the envelope must not nest it.
    assert_eq!(json["clientId"], "konobangu-webui");
    assert_eq!(json["redirectUrl"], "https://ui.konobangu.test/auth/callback");
    assert_eq!(json["issuerUrl"], "https://idp.example/");
    assert_eq!(json["wellKnownUrl"], "https://idp.example/.well-known/openid-configuration");
    assert_eq!(json["pkceEnabled"], true);
    assert_eq!(json["generatedAt"], 1_700_000_000_000u64);
    assert_eq!(json["scopes"][2], "konobangu:api");
    assert_eq!(json["provider"], "oidc");
    assert_eq!(json["resource"]["audience"], "urn:konobangu:api");
    assert_eq!(json["resource"]["requiredScopes"][0], "konobangu:api");
    // The unsafe fields are absent under the names the browser would read.
    for unsafe_key in ["clientSecret", "claimsCheckScript", "client_secret", "claims_check_script"] {
      assert!(json.get(unsafe_key).is_none(), "{unsafe_key} must never be projected");
    }
  }
}
