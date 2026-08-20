//! OIDC configuration and verification shared by bundled and external issuers.
use std::sync::Arc;

use konobangu_auth_provider::ProviderRuntime;
use securitydept_token_set_context::frontend_oidc_mode::FrontendOidcModeConfigProjection;

use super::{AuthError, config::OidcAuthConfig, projection::oidc_projection_template, resource::ResourceVerifier};

pub struct OidcAuthService {
  pub verifier: ResourceVerifier,
  pub projection: FrontendOidcModeConfigProjection,
  pub resource_audience: String,
  pub required_scopes: Vec<String>,
}

impl OidcAuthService {
  pub async fn new(config: &OidcAuthConfig, bundled: Option<Arc<ProviderRuntime>>) -> Result<Self, AuthError> {
    let projection = oidc_projection_template(config)?;
    let verifier = match bundled {
      Some(provider) => ResourceVerifier::bundled(provider),
      None => ResourceVerifier::new(config).await?,
    };
    Ok(Self {
      verifier,
      projection,
      resource_audience: config.resource.audience.clone(),
      required_scopes: config.resource.required_scopes.clone(),
    })
  }
}
