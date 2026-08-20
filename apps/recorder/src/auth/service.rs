use std::sync::Arc;

use axum::http::request::Parts;
use http::header::HeaderValue;
use konobangu_auth_provider::ProviderRuntime;
use konobangu_database::PoolConfig;
use sea_orm::DatabaseConnection;

use super::{
  AuthConfig,
  config::{ApplicationAdministrator, OidcAuthConfig, OidcBrowserConfig, OidcResourceConfig, ProviderAuthConfig},
  connect_identity_db,
  errors::AuthError,
  oidc::OidcAuthService,
  resource::bearer_credential,
  webui::WebuiPolicy,
};

#[derive(Clone, Debug)]
pub struct AuthUserInfo {
  pub subscriber_auth: crate::models::auth::Model,
  pub is_admin: bool,
}

/// The API always verifies OIDC access tokens. Only issuer construction varies.
pub struct AuthService {
  pub webui: WebuiPolicy,
  identity_db: DatabaseConnection,
  pub resource_grants: super::native_resources::NativeResourceGrants,
  pub oidc: OidcAuthService,
  bundled: Option<Arc<ProviderRuntime>>,
  administrators: Vec<ApplicationAdministrator>,
}

impl AuthService {
  pub async fn from_conf(config: AuthConfig) -> Result<Self, AuthError> {
    let webui = config.webui_policy()?;
    let resource_grants = super::native_resources::NativeResourceGrants::new(&config.resource_grants)?;
    let identity_db = connect_identity_db(&config.identity.database_url).await?;
    let bundled = match &config.provider {
      ProviderAuthConfig::Bundled(provider) => match ProviderRuntime::new(
        (**provider).clone(),
        PoolConfig {
          max_connections: 3,
          ..Default::default()
        },
      )
      .await
      {
        Ok(provider) => Some(provider),
        Err(_) => {
          let _ = identity_db.close().await;
          return Err(AuthError::Configuration {
            message: "Bundled OIDC provider initialization failed (details redacted)".into(),
          });
        }
      },
      ProviderAuthConfig::Oidc(_) => None,
    };
    let oidc_config = match &config.provider {
      ProviderAuthConfig::Oidc(config) => config.clone(),
      ProviderAuthConfig::Bundled(config) => OidcAuthConfig {
        issuer: config.issuer.clone(),
        trusted_ca_file: None,
        browser: OidcBrowserConfig {
          client_id: config.client.client_id.clone(),
          scopes: config.client.scopes.clone(),
        },
        resource: OidcResourceConfig {
          audience: config.resource.audience.clone(),
          required_scopes: config.resource.required_scopes.clone(),
        },
      },
    };
    let oidc = match OidcAuthService::new(&oidc_config, bundled.clone()).await {
      Ok(oidc) => oidc,
      Err(error) => {
        if let Some(provider) = &bundled {
          let _ = provider.close().await;
        }
        let _ = identity_db.close().await;
        return Err(error);
      }
    };
    Ok(Self {
      webui,
      identity_db,
      oidc,
      bundled,
      resource_grants,
      administrators: config.administrators,
    })
  }

  pub fn identity_db(&self) -> &DatabaseConnection {
    &self.identity_db
  }

  pub fn bundled_provider(&self) -> Option<&Arc<ProviderRuntime>> {
    self.bundled.as_ref()
  }

  pub async fn close(&self) -> Result<(), AuthError> {
    // Complete both closures even when one pool reports an error.
    let provider = if let Some(provider) = &self.bundled { provider.close().await } else { Ok(()) };
    let identity = self.identity_db.clone().close().await;
    provider.and(identity).map_err(|_| AuthError::IdentityStorage)
  }

  pub fn public_config(&self, headers: &http::HeaderMap, generated_at_ms: u64) -> Result<super::PublicAuthConfig, http::StatusCode> {
    let source = self.webui.resolve_source(headers)?;
    if let Some(provider) = &self.bundled
      && !provider.config().client.redirect_uris.contains(&source.callback_url())
    {
      return Err(http::StatusCode::FORBIDDEN);
    }
    Ok(super::PublicAuthConfig {
      provider: super::PublicAuthProvider::Oidc,
      resource: super::projection::PublicResourceConfig {
        audience: self.oidc.resource_audience.clone(),
        required_scopes: self.oidc.required_scopes.clone(),
      },
      projection: super::projection::stamped(&self.oidc.projection, &source, generated_at_ms),
    })
  }
}

impl AuthService {
  #[tracing::instrument(skip(self, request))]
  pub async fn extract_user_info(&self, request: &mut Parts) -> Result<AuthUserInfo, AuthError> {
    let authorization = request.headers.get(http::header::AUTHORIZATION).and_then(|value| value.to_str().ok());
    let principal = self.oidc.verifier.authenticate(bearer_credential(authorization)?).await?;
    let bundled_administrator = self
      .bundled
      .as_ref()
      .is_some_and(|provider| provider.config().issuer == principal.issuer && provider.config().credentials.subject == principal.subject);
    let subscriber_auth =
      crate::models::auth::Model::find_or_create_oidc_with_seed(&self.identity_db, &principal.issuer, &principal.subject, bundled_administrator)
        .await
        .map_err(|_| AuthError::FindAuthRecordError)?;
    let is_admin = bundled_administrator
      || self
        .administrators
        .iter()
        .any(|admin| admin.issuer == principal.issuer && admin.subject == principal.subject);
    Ok(AuthUserInfo { subscriber_auth, is_admin })
  }

  pub fn www_authenticate_header_value(&self) -> Option<HeaderValue> {
    Some(HeaderValue::from_static("Bearer"))
  }
}
