//! Local access-token verification for this API.
//!
//! External issuers use SDK discovery and JWKS caching; the bundled issuer
//! uses its in-memory public keys. Both apply the same token-use projection.
use std::sync::Arc;

use futures::future::BoxFuture;
use konobangu_auth_provider::ProviderRuntime;
pub use konobangu_auth_provider::ResourcePrincipal as ResourceTokenPrincipal;
use securitydept_creds::TokenData;
use securitydept_oauth_resource_server::{OAuthResourceServerConfig, OAuthResourceServerVerifier};

use super::{AuthError, config::OidcAuthConfig};

pub enum ResourceVerifier {
  External(Box<OAuthResourceServerVerifier>),
  Bundled(Arc<ProviderRuntime>),
}

impl ResourceVerifier {
  /// Resolves discovery once at startup. A failure here is fatal: there is no
  /// fallback to cookies or Basic for an OIDC deployment.
  pub async fn new(config: &OidcAuthConfig) -> Result<Self, AuthError> {
    let mut server = OAuthResourceServerConfig::default();
    let discovery_base = config.issuer.trim_end_matches('/');
    server.remote.issuer_url = Some(config.issuer.clone());
    server.remote.well_known_url = Some(format!("{discovery_base}/.well-known/openid-configuration"));
    server.audiences = vec![config.resource.audience.clone()];
    server.required_scopes = config.resource.required_scopes.clone();
    server.clock_skew = std::time::Duration::from_secs(konobangu_auth_provider::ACCESS_TOKEN_CLOCK_SKEW_SECONDS);
    let mut http = fetch::reqwest::Client::builder();
    if let Some(path) = config.trusted_ca_file.as_deref() {
      let invalid = || AuthError::Configuration {
        message: "auth.provider.trusted_ca_file must be a readable PEM certificate bundle".into(),
      };
      let roots = fetch::reqwest::Certificate::from_pem_bundle(&std::fs::read(path).map_err(|_| invalid())?).map_err(|_| invalid())?;
      http = http.tls_certs_merge(roots);
    }
    let verifier = OAuthResourceServerVerifier::from_config_with_http_client_builder(server, http)
      .await
      .map_err(|_| AuthError::Configuration {
        message: "OIDC discovery or resource-server configuration failed; no fallback authentication is enabled".into(),
      })?;
    Ok(Self::External(Box::new(verifier)))
  }

  pub fn bundled(provider: Arc<ProviderRuntime>) -> Self {
    Self::Bundled(provider)
  }

  /// Verifies signature, trusted issuer, audience, expiry, not-before and the
  /// required scopes through the SDK, then applies our own token-use and
  /// signature-family checks before projecting the principal.
  pub fn authenticate<'a>(&'a self, bearer: &'a str) -> BoxFuture<'a, Result<ResourceTokenPrincipal, AuthError>> {
    // Bound the composed middleware future independently of SDK internals.
    Box::pin(async move {
      let verifier = match self {
        Self::External(verifier) => verifier,
        Self::Bundled(provider) => {
          return provider.verify_access_token(bearer).map_err(|_| AuthError::ResourceTokenInvalid);
        }
      };
      let verified = verifier
        .verify_rfc9068_access_token(bearer)
        .await
        .map_err(|_| AuthError::ResourceTokenInvalid)?;
      let TokenData::JWT(data) = &verified.token_data else {
        // Opaque and encrypted tokens are out of scope: this deployment does
        // not configure introspection.
        return Err(AuthError::ResourceTokenInvalid);
      };
      konobangu_auth_provider::project_verified_access_token(data).map_err(|_| AuthError::ResourceTokenUse)
    })
  }
}

/// Extracts a bearer credential. A Basic credential must not be accepted on an
/// OIDC deployment's resource path.
pub fn bearer_credential(authorization: Option<&str>) -> Result<&str, AuthError> {
  let value = authorization.ok_or(AuthError::ResourceTokenMissing)?;
  let (scheme, token) = value.split_once(' ').ok_or(AuthError::ResourceTokenMissing)?;
  if !scheme.eq_ignore_ascii_case("Bearer") {
    return Err(AuthError::ResourceTokenMissing);
  }
  let token = token.trim();
  if token.is_empty() {
    return Err(AuthError::ResourceTokenMissing);
  }
  Ok(token)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_only_a_nonempty_bearer_credential() {
    assert_eq!(bearer_credential(Some("Bearer abc")).unwrap(), "abc");
    assert_eq!(bearer_credential(Some("bearer abc")).unwrap(), "abc");
    for value in [None, Some("abc"), Some("Basic dXNlcjpwdw=="), Some("Bearer "), Some("Bearer")] {
      assert!(
        matches!(bearer_credential(value), Err(AuthError::ResourceTokenMissing)),
        "{value:?} must be rejected"
      );
    }
  }
}
