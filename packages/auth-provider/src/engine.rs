use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use openidconnect::{OAuth2TokenResponse, RefreshToken, core::CoreTokenResponse};
use oxide_auth::{
  code_grant::{accesstoken, authorization, refresh},
  frontends::simple::extensions::{AccessTokenAddon, AddonList, AddonResult, AuthorizationAddon, Pkce},
  primitives::{
    grant::{Grant, GrantExtension, Value},
    issuer::{IssuedToken, RefreshedToken, TokenType},
    registrar::{BoundClient, ClientUrl, PreGrant, RegisteredUrl, Registrar, RegistrarError},
    scope::Scope,
  },
};
use oxide_auth_async::{code_grant as flow, primitives};
use serde::{Deserialize, Serialize};

use crate::{
  ProviderRuntime,
  config::ClientConfig,
  store::{OidcData, StoredGrant},
};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthorizationRequest {
  pub client_id: Option<String>,
  pub redirect_uri: Option<String>,
  pub response_type: Option<String>,
  pub scope: Option<String>,
  pub state: Option<String>,
  pub nonce: Option<String>,
  pub code_challenge: Option<String>,
  pub code_challenge_method: Option<String>,
  pub prompt: Option<String>,
  pub max_age: Option<String>,
  pub response_mode: Option<String>,
  pub request: Option<String>,
  pub request_uri: Option<String>,
}
impl authorization::Request for AuthorizationRequest {
  fn valid(&self) -> bool {
    self.prompt.is_none()
      && self.max_age.is_none()
      && self.response_mode.as_deref().is_none_or(|value| value == "query")
      && self.request.is_none()
      && self.request_uri.is_none()
      && self.code_challenge_method.as_deref() == Some("S256")
      && self
        .code_challenge
        .as_ref()
        .is_some_and(|value| value.len() == 43 && value.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-')))
  }
  fn client_id(&self) -> Option<Cow<'_, str>> {
    self.client_id.as_deref().map(Cow::Borrowed)
  }
  fn redirect_uri(&self) -> Option<Cow<'_, str>> {
    self.redirect_uri.as_deref().map(Cow::Borrowed)
  }
  fn response_type(&self) -> Option<Cow<'_, str>> {
    self.response_type.as_deref().map(Cow::Borrowed)
  }
  fn scope(&self) -> Option<Cow<'_, str>> {
    self.scope.as_deref().map(Cow::Borrowed)
  }
  fn state(&self) -> Option<Cow<'_, str>> {
    self.state.as_deref().map(Cow::Borrowed)
  }
  fn extension(&self, key: &str) -> Option<Cow<'_, str>> {
    match key {
      "nonce" => self.nonce.as_deref(),
      "code_challenge" => self.code_challenge.as_deref(),
      "code_challenge_method" => self.code_challenge_method.as_deref(),
      _ => None,
    }
    .map(Cow::Borrowed)
  }
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TokenRequest {
  pub grant_type: Option<String>,
  pub client_id: Option<String>,
  pub code: Option<String>,
  pub redirect_uri: Option<String>,
  pub code_verifier: Option<String>,
  pub refresh_token: Option<String>,
  pub scope: Option<String>,
}
impl TokenRequest {
  pub(crate) fn well_formed(&self) -> bool {
    self
      .code_verifier
      .as_ref()
      .is_none_or(|value| (43..=128).contains(&value.len()) && value.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'.' | b'_' | b'~')))
  }
}
impl accesstoken::Request for TokenRequest {
  fn valid(&self) -> bool {
    self.well_formed()
  }
  fn code(&self) -> Option<Cow<'_, str>> {
    self.code.as_deref().map(Cow::Borrowed)
  }
  fn authorization(&self) -> accesstoken::Authorization<'_> {
    accesstoken::Authorization::None
  }
  fn client_id(&self) -> Option<Cow<'_, str>> {
    self.client_id.as_deref().map(Cow::Borrowed)
  }
  fn redirect_uri(&self) -> Option<Cow<'_, str>> {
    self.redirect_uri.as_deref().map(Cow::Borrowed)
  }
  fn grant_type(&self) -> Option<Cow<'_, str>> {
    self.grant_type.as_deref().map(Cow::Borrowed)
  }
  fn extension(&self, key: &str) -> Option<Cow<'_, str>> {
    (key == "code_verifier").then_some(self.code_verifier.as_deref()).flatten().map(Cow::Borrowed)
  }
}
impl refresh::Request for TokenRequest {
  fn valid(&self) -> bool {
    self.well_formed()
  }
  fn refresh_token(&self) -> Option<Cow<'_, str>> {
    self.refresh_token.as_deref().map(Cow::Borrowed)
  }
  fn scope(&self) -> Option<Cow<'_, str>> {
    self.scope.as_deref().map(Cow::Borrowed)
  }
  fn grant_type(&self) -> Option<Cow<'_, str>> {
    self.grant_type.as_deref().map(Cow::Borrowed)
  }
  fn authorization(&self) -> Option<(Cow<'_, str>, Cow<'_, [u8]>)> {
    None
  }
  fn extension(&self, key: &str) -> Option<Cow<'_, str>> {
    (key == "client_id").then_some(self.client_id.as_deref()).flatten().map(Cow::Borrowed)
  }
}

struct FixedClient(ClientConfig);
impl Registrar for FixedClient {
  fn bound_redirect<'a>(&self, bound: ClientUrl<'a>) -> Result<BoundClient<'a>, RegistrarError> {
    let redirect = bound.redirect_uri.ok_or(RegistrarError::Unspecified)?;
    if bound.client_id != self.0.client_id || !self.0.redirect_uris.iter().any(|uri| uri == redirect.as_str()) {
      return Err(RegistrarError::Unspecified);
    }
    Ok(BoundClient {
      client_id: bound.client_id,
      redirect_uri: Cow::Owned(RegisteredUrl::Exact(redirect.into_owned())),
    })
  }
  fn negotiate(&self, client: BoundClient<'_>, scope: Option<Scope>) -> Result<PreGrant, RegistrarError> {
    let scope = scope.ok_or(RegistrarError::Unspecified)?;
    if !scope.iter().any(|value| value == "openid") || !scope.iter().all(|value| self.0.scopes.iter().any(|allowed| allowed == value)) {
      return Err(RegistrarError::Unspecified);
    }
    Ok(PreGrant {
      client_id: client.client_id.into_owned(),
      redirect_uri: client.redirect_uri.into_owned(),
      scope,
    })
  }
  fn check(&self, id: &str, pass: Option<&[u8]>) -> Result<(), RegistrarError> {
    if id == self.0.client_id && pass.is_none() {
      Ok(())
    } else {
      Err(RegistrarError::Unspecified)
    }
  }
}
struct OidcAddon;
impl GrantExtension for OidcAddon {
  fn identifier(&self) -> &'static str {
    "konobangu_oidc"
  }
}
impl AuthorizationAddon for OidcAddon {
  fn execute(&self, request: &dyn authorization::Request) -> AddonResult {
    match serde_json::to_string(&OidcData {
      nonce: request.extension("nonce").map(Cow::into_owned),
      auth_time: Utc::now().timestamp(),
    }) {
      Ok(data) => AddonResult::Data(Value::private(Some(data))),
      Err(_) => AddonResult::Err,
    }
  }
}
impl AccessTokenAddon for OidcAddon {
  fn execute(&self, _: &dyn accesstoken::Request, data: Option<Value>) -> AddonResult {
    match data {
      Some(data) => AddonResult::Data(data),
      None => AddonResult::Err,
    }
  }
}
struct Codes(Arc<ProviderRuntime>, Option<String>, bool);
#[async_trait]
impl primitives::Authorizer for Codes {
  async fn authorize(&mut self, mut grant: Grant) -> Result<String, ()> {
    grant.until = Utc::now() + Duration::seconds(self.0.config.tokens.authorization_code_ttl_seconds as i64);
    let mut data = StoredGrant::from_grant(grant).map_err(|_| ())?;
    if let Some(redirect) = &self.1 {
      data.redirect_uri = redirect.clone();
    }
    self.0.store.put("code", &data, data.until).await.map_err(|_| ())
  }
  async fn extract(&mut self, token: &str) -> Result<Option<Grant>, ()> {
    self.2 = true;
    let grant = self.0.store.take::<StoredGrant>(token, "code").await.map_err(|_| ())?;
    grant
      .filter(|grant| self.1.as_ref().is_some_and(|redirect| *redirect == grant.redirect_uri))
      .map(|grant| grant.to_grant().map_err(|_| ()))
      .transpose()
  }
}
struct Tokens {
  runtime: Arc<ProviderRuntime>,
  response: Option<CoreTokenResponse>,
  invalid_grant: bool,
}
#[async_trait]
impl primitives::Issuer for Tokens {
  async fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
    let data = StoredGrant::from_grant(grant).map_err(|_| ())?;
    let mut response = self.runtime.signer.mint(&self.runtime.config, &data, None).map_err(|_| ())?;
    let refresh = if data.scope.split_whitespace().any(|scope| scope == "offline_access") {
      Some(
        self
          .runtime
          .store
          .issue_refresh(&data, Utc::now().timestamp() + self.runtime.config.tokens.refresh_token_ttl_seconds as i64)
          .await
          .map_err(|_| ())?,
      )
    } else {
      None
    };
    response.set_refresh_token(refresh.clone().map(RefreshToken::new));
    let issued = IssuedToken {
      token: response.access_token().secret().clone(),
      refresh,
      until: Utc::now() + Duration::seconds(self.runtime.config.tokens.access_token_ttl_seconds as i64),
      token_type: TokenType::Bearer,
    };
    self.response = Some(response);
    Ok(issued)
  }
  async fn refresh(&mut self, token: &str, grant: Grant) -> Result<RefreshedToken, ()> {
    let data = StoredGrant::from_grant(grant).map_err(|_| ())?;
    let mut response = self.runtime.signer.mint(&self.runtime.config, &data, None).map_err(|_| ())?;
    let next = match self.runtime.store.rotate_refresh(token, &data).await.map_err(|_| ())? {
      Some(token) => token,
      None => {
        self.invalid_grant = true;
        return Err(());
      }
    };
    response.set_refresh_token(Some(RefreshToken::new(next.clone())));
    let refreshed = RefreshedToken {
      token: response.access_token().secret().clone(),
      refresh: Some(next),
      until: Utc::now() + Duration::seconds(self.runtime.config.tokens.access_token_ttl_seconds as i64),
      token_type: TokenType::Bearer,
    };
    self.response = Some(response);
    Ok(refreshed)
  }
  async fn recover_token(&mut self, _: &str) -> Result<Option<Grant>, ()> {
    Ok(None)
  }
  async fn recover_refresh(&mut self, token: &str) -> Result<Option<Grant>, ()> {
    self
      .runtime
      .store
      .recover_refresh(token)
      .await
      .map_err(|_| ())?
      .map(|grant| grant.to_grant().map_err(|_| ()))
      .transpose()
  }
}

pub(crate) struct Flow {
  client: FixedClient,
  codes: Codes,
  tokens: Tokens,
  addons: AddonList,
}
impl Flow {
  pub fn new(runtime: Arc<ProviderRuntime>) -> Self {
    let mut addons = AddonList::new();
    addons.push_code(Pkce::required());
    addons.push_code(OidcAddon);
    Self {
      client: FixedClient(runtime.config.client.clone()),
      codes: Codes(runtime.clone(), None, false),
      tokens: Tokens {
        runtime,
        response: None,
        invalid_grant: false,
      },
      addons,
    }
  }
  pub fn bind_redirect(&mut self, redirect: Option<String>) {
    self.codes.1 = redirect;
  }
  pub fn code_queried(&self) -> bool {
    self.codes.2
  }
  pub fn invalid_grant(&self) -> bool {
    self.tokens.invalid_grant
  }
  pub fn take_response(&mut self) -> Option<CoreTokenResponse> {
    self.tokens.response.take()
  }
}
impl flow::authorization::Endpoint for Flow {
  fn registrar(&self) -> &(dyn primitives::Registrar + Sync) {
    &self.client
  }
  fn authorizer(&mut self) -> &mut (dyn primitives::Authorizer + Send) {
    &mut self.codes
  }
  fn extension(&mut self) -> &mut (dyn flow::authorization::Extension + Send) {
    &mut self.addons
  }
}
impl flow::access_token::Endpoint for Flow {
  fn registrar(&self) -> &(dyn primitives::Registrar + Sync) {
    &self.client
  }
  fn authorizer(&mut self) -> &mut (dyn primitives::Authorizer + Send) {
    &mut self.codes
  }
  fn issuer(&mut self) -> &mut (dyn primitives::Issuer + Send) {
    &mut self.tokens
  }
  fn extension(&mut self) -> &mut (dyn flow::access_token::Extension + Send) {
    &mut self.addons
  }
}
impl flow::refresh::Endpoint for Flow {
  fn registrar(&self) -> &(dyn primitives::Registrar + Sync) {
    &self.client
  }
  fn issuer(&mut self) -> &mut (dyn primitives::Issuer + Send) {
    &mut self.tokens
  }
}
