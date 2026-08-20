//! A bounded first-party OpenID Provider. Application identities and
//! permissions remain in the resource server; this crate owns only
//! authorization state.
mod config;
mod crypto;
mod engine;
mod http;
pub mod migration;
mod role;
mod schema;
mod store;

use std::sync::Arc;

pub use config::*;
pub use schema::schema_name as provider_schema_name;

/// Shared resource-server clock tolerance for bundled and external issuers.
pub const ACCESS_TOKEN_CLOCK_SKEW_SECONDS: u64 = 60;
use konobangu_database::PoolConfig;
pub use migration::{apply_migrations, grant_access, revert_migrations};
use openidconnect::{
  AuthUrl, EmptyAdditionalProviderMetadata, IssuerUrl, JsonWebKeySetUrl, ResponseTypes, Scope, TokenUrl, UserInfoUrl,
  core::{CoreClientAuthMethod, CoreGrantType, CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType},
};
use securitydept_core::{
  basic_auth_context::{BasicAuthContext, BasicAuthContextConfig, BasicAuthContextConfigSource, BasicAuthZoneConfig},
  creds::{Argon2BasicAuthCred, BasicAuthCredsConfig, JwtTokenData, TokenJwtClaims},
  realip::RealIpResolver,
};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum ProviderError {
  #[snafu(display("Bundled provider configuration is invalid"))]
  Configuration,
  #[snafu(display("Bundled provider storage operation failed"))]
  Storage,
  #[snafu(display("Bundled provider signing operation failed"))]
  Signing,
  #[snafu(display("Bundled provider protocol operation failed"))]
  Protocol,
  #[snafu(display("Access token is invalid"))]
  InvalidToken,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourcePrincipal {
  pub subject: String,
  pub issuer: String,
  pub audiences: Vec<String>,
  pub scopes: Vec<String>,
  pub client_id: Option<String>,
}

/// Both bundled and remote issuers pass through this post-verification policy.
/// The caller must first verify signature, issuer, audience, time and scopes.
pub fn project_verified_access_token(data: &JwtTokenData<TokenJwtClaims>) -> Result<ResourcePrincipal, ProviderError> {
  if !data
    .header
    .typ
    .as_deref()
    .is_some_and(|typ| ["at+jwt", "application/at+jwt"].iter().any(|allowed| typ.eq_ignore_ascii_case(allowed)))
    || matches!(
      data.header.alg,
      jsonwebtoken::Algorithm::HS256 | jsonwebtoken::Algorithm::HS384 | jsonwebtoken::Algorithm::HS512
    )
    || data.claims.subject.trim().is_empty()
  {
    return Err(ProviderError::InvalidToken);
  }
  let claims = &data.claims;
  Ok(ResourcePrincipal {
    subject: claims.subject.clone(),
    issuer: claims.issuer.clone(),
    audiences: claims.audience.to_vec(),
    scopes: claims.scope.as_ref().map(|value| value.to_vec()).unwrap_or_default(),
    client_id: claims.client_id.clone(),
  })
}

#[derive(Clone, Serialize)]
pub struct DiscoveryMetadata {
  #[serde(flatten)]
  pub core: CoreProviderMetadata,
  pub code_challenge_methods_supported: [&'static str; 1],
}
pub struct ProviderRuntime {
  config: ProviderConfig,
  store: store::Store,
  signer: crypto::Signer,
  basic: Arc<BasicAuthContext<Argon2BasicAuthCred>>,
  real_ip: RealIpResolver,
  basic_budget: Arc<tokio::sync::Semaphore>,
}
impl ProviderRuntime {
  pub async fn new(config: ProviderConfig, pool: PoolConfig) -> Result<Arc<Self>, ProviderError> {
    config.validate()?;
    let signer = crypto::Signer::new(&config)?;
    let credentials =
      Argon2BasicAuthCred::new(config.credentials.username.clone(), config.credentials.password.clone()).map_err(|_| ProviderError::Configuration)?;
    let mut basic = BasicAuthContextConfig::builder()
      .creds(BasicAuthCredsConfig { users: vec![credentials] })
      .zones(vec![
        BasicAuthZoneConfig::builder()
          .zone_prefix("/auth-provider".to_owned())
          .login_subpath("/login".to_owned())
          .realm("Konobangu Provider".to_owned())
          .build(),
      ])
      .build();
    basic.real_ip_access = config.real_ip.as_ref().and_then(|value| value.access.clone());
    let basic = BasicAuthContext::from_resolved_config(BasicAuthContextConfigSource::resolve_all(&basic).map_err(|_| ProviderError::Configuration)?)
      .map_err(|_| ProviderError::Configuration)?;
    let real_ip = RealIpResolver::from_config(config.real_ip.as_ref().map(|value| value.resolve.clone()).unwrap_or_default())
      .await
      .map_err(|_| ProviderError::Configuration)?;
    // Open resources last so every earlier construction failure has no pool to
    // close.
    let db = konobangu_database::connect(&config.database.url, &pool, Some(konobangu_database::roles::AUTH_PROVIDER_ACCESS_ROLE))
      .await
      .map_err(|_| ProviderError::Storage)?;
    if let Err(error) = role::validate(&db).await {
      let _ = db.close().await;
      return Err(error);
    }
    Ok(Arc::new(Self {
      config: config.clone(),
      store: store::Store {
        db,
        issuer: config.issuer.clone(),
      },
      signer,
      basic: Arc::new(basic),
      real_ip,
      basic_budget: Arc::new(tokio::sync::Semaphore::new(2)),
    }))
  }
  pub fn config(&self) -> &ProviderConfig {
    &self.config
  }
  pub fn router(self: Arc<Self>) -> axum::Router {
    http::router(self)
  }
  pub fn jwks(&self) -> openidconnect::core::CoreJsonWebKeySet {
    self.signer.jwks()
  }
  pub fn metadata(&self) -> DiscoveryMetadata {
    let config = &self.config;
    DiscoveryMetadata {
      core: CoreProviderMetadata::new(
        IssuerUrl::new(config.issuer.clone()).expect("validated issuer"),
        AuthUrl::new(config.endpoint("authorize")).expect("validated issuer"),
        JsonWebKeySetUrl::new(config.endpoint("jwks")).expect("validated issuer"),
        vec![ResponseTypes::new(vec![CoreResponseType::Code])],
        vec![CoreSubjectIdentifierType::Public],
        vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256],
        EmptyAdditionalProviderMetadata {},
      )
      .set_token_endpoint(Some(TokenUrl::new(config.endpoint("token")).expect("validated issuer")))
      .set_userinfo_endpoint(Some(UserInfoUrl::new(config.endpoint("userinfo")).expect("validated issuer")))
      .set_scopes_supported(Some(config.client.scopes.iter().cloned().map(Scope::new).collect()))
      .set_grant_types_supported(Some(vec![CoreGrantType::AuthorizationCode, CoreGrantType::RefreshToken]))
      .set_response_modes_supported(Some(vec![CoreResponseMode::Query]))
      .set_token_endpoint_auth_methods_supported(Some(vec![CoreClientAuthMethod::None])),
      code_challenge_methods_supported: ["S256"],
    }
  }
  pub fn verify_access_token(&self, token: &str) -> Result<ResourcePrincipal, ProviderError> {
    self.signer.verify(&self.config, token)
  }
  pub async fn close(&self) -> Result<(), sea_orm::DbErr> {
    self.store.db.close_by_ref().await
  }
}

#[cfg(test)]
mod tests {
  use jsonwebtoken::{Algorithm, Header};

  use super::*;
  fn verified(typ: Option<&str>, algorithm: Algorithm, subject: &str) -> JwtTokenData<TokenJwtClaims> {
    let mut header = Header::new(algorithm);
    header.typ = typ.map(str::to_owned);
    JwtTokenData { header,claims:serde_json::from_value(serde_json::json!({"iss":"https://issuer.example/auth-provider","sub":subject,"aud":"urn:konobangu:api","exp":9999999999_u64,"iat":1,"jti":"id","scope":"konobangu:api","client_id":"webui"})).unwrap() }
  }
  #[test]
  fn unified_verified_token_projection_requires_access_type_asymmetric_signature_and_subject() {
    for typ in ["at+jwt", "AT+JWT", "application/at+jwt", "APPLICATION/AT+JWT"] {
      assert!(project_verified_access_token(&verified(Some(typ), Algorithm::RS256, "owner")).is_ok());
    }
    for typ in [None, Some("JWT"), Some("id+jwt"), Some(""), Some("refresh")] {
      assert!(project_verified_access_token(&verified(typ, Algorithm::RS256, "owner")).is_err());
    }
    for algorithm in [Algorithm::HS256, Algorithm::HS384, Algorithm::HS512] {
      assert!(project_verified_access_token(&verified(Some("at+jwt"), algorithm, "owner")).is_err());
    }
    for subject in ["", " "] {
      assert!(project_verified_access_token(&verified(Some("at+jwt"), Algorithm::RS256, subject)).is_err());
    }
  }
}
