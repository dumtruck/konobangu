use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use openidconnect::{
  AccessToken, Audience, EmptyAdditionalClaims, IdTokenFields, IssuerUrl, JsonWebKeyId, Nonce, PrivateSigningKey, RefreshToken, Scope, StandardClaims,
  SubjectIdentifier,
  core::{CoreIdToken, CoreIdTokenClaims, CoreJsonWebKeySet, CoreJwsSigningAlgorithm, CoreRsaPrivateSigningKey, CoreTokenResponse, CoreTokenType},
};
use rsa::{
  RsaPrivateKey,
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey},
  pkcs8::DecodePrivateKey,
  traits::PublicKeyParts,
};
use securitydept_core::creds::{TokenJwtClaims, verify_token_jwt_with_jwks};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{ProviderConfig, ProviderError, ResourcePrincipal, store::StoredGrant};

pub struct Signer {
  key: CoreRsaPrivateSigningKey,
  encoding: EncodingKey,
  jwks: CoreJsonWebKeySet,
  verify_jwks: jsonwebtoken::jwk::JwkSet,
  kid: String,
}
#[derive(Serialize)]
struct AccessClaims<'a> {
  iss: &'a str,
  sub: &'a str,
  aud: &'a str,
  iat: i64,
  exp: i64,
  nbf: i64,
  jti: String,
  client_id: &'a str,
  scope: String,
}
impl Signer {
  pub fn new(config: &ProviderConfig) -> Result<Self, ProviderError> {
    let pem = &config.signing.private_key_pem;
    let key = RsaPrivateKey::from_pkcs8_pem(pem)
      .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem))
      .map_err(|_| ProviderError::Configuration)?;
    if key.n().bits() < 2048 {
      return Err(ProviderError::Configuration);
    }
    let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(key.n().to_bytes_be()));
    let normalized = key.to_pkcs1_pem(Default::default()).map_err(|_| ProviderError::Configuration)?;
    let key = CoreRsaPrivateSigningKey::from_pem(&normalized, Some(JsonWebKeyId::new(kid.clone()))).map_err(|_| ProviderError::Configuration)?;
    let jwks = CoreJsonWebKeySet::new(vec![key.as_verification_key()]);
    let encoding = EncodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| ProviderError::Configuration)?;
    let verify_jwks =
      serde_json::from_value(serde_json::to_value(&jwks).map_err(|_| ProviderError::Configuration)?).map_err(|_| ProviderError::Configuration)?;
    Ok(Self {
      key,
      encoding,
      jwks,
      verify_jwks,
      kid,
    })
  }
  pub fn jwks(&self) -> CoreJsonWebKeySet {
    self.jwks.clone()
  }
  pub fn mint(&self, config: &ProviderConfig, grant: &StoredGrant, refresh: Option<String>) -> Result<CoreTokenResponse, ProviderError> {
    let now = Utc::now();
    let expires = now + Duration::seconds(config.tokens.access_token_ttl_seconds as i64);
    let mut header = Header::new(Algorithm::RS256);
    header.typ = Some("at+jwt".into());
    header.kid = Some(self.kid.clone());
    let access = jsonwebtoken::encode(
      &header,
      &AccessClaims {
        iss: &config.issuer,
        sub: &grant.owner_id,
        aud: &config.resource.audience,
        iat: now.timestamp(),
        exp: expires.timestamp(),
        nbf: now.timestamp(),
        jti: uuid::Uuid::new_v4().to_string(),
        client_id: &grant.client_id,
        scope: grant.scope.clone(),
      },
      &self.encoding,
    )
    .map_err(|_| ProviderError::Signing)?;
    let access = AccessToken::new(access);
    let mut claims = CoreIdTokenClaims::new(
      IssuerUrl::new(config.issuer.clone()).map_err(|_| ProviderError::Configuration)?,
      vec![Audience::new(grant.client_id.clone())],
      expires,
      now,
      StandardClaims::new(SubjectIdentifier::new(grant.owner_id.clone())),
      EmptyAdditionalClaims {},
    );
    claims = claims
      .set_nonce(grant.nonce.clone().map(Nonce::new))
      .set_auth_time(chrono::DateTime::from_timestamp(grant.auth_time, 0));
    let id_token =
      CoreIdToken::new(claims, &self.key, CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256, Some(&access), None).map_err(|_| ProviderError::Signing)?;
    let mut response = CoreTokenResponse::new(
      access,
      CoreTokenType::Bearer,
      IdTokenFields::new(Some(id_token), openidconnect::EmptyExtraTokenFields {}),
    );
    response.set_expires_in(Some(&std::time::Duration::from_secs(config.tokens.access_token_ttl_seconds)));
    response.set_scopes(Some(grant.scope.split_whitespace().map(|scope| Scope::new(scope.to_owned())).collect()));
    response.set_refresh_token(refresh.map(RefreshToken::new));
    Ok(response)
  }
  pub fn verify(&self, config: &ProviderConfig, token: &str) -> Result<ResourcePrincipal, ProviderError> {
    self.verify_scopes(config, token, &config.resource.required_scopes)
  }
  pub fn verify_userinfo(&self, config: &ProviderConfig, token: &str) -> Result<ResourcePrincipal, ProviderError> {
    self.verify_scopes(config, token, &["openid".to_owned()])
  }
  fn verify_scopes(&self, config: &ProviderConfig, token: &str, required_scopes: &[String]) -> Result<ResourcePrincipal, ProviderError> {
    let data = verify_token_jwt_with_jwks::<TokenJwtClaims, _, _, _>(token, &self.verify_jwks, |mut validation| {
      validation.algorithms = vec![Algorithm::RS256];
      validation.set_issuer(&[&config.issuer]);
      validation.set_audience(&[&config.resource.audience]);
      validation.validate_nbf = true;
      validation.leeway = crate::ACCESS_TOKEN_CLOCK_SKEW_SECONDS;
      validation.set_required_spec_claims(&["iss", "sub", "aud", "exp", "iat", "jti"]);
      Ok(validation)
    })
    .map_err(|_| ProviderError::InvalidToken)?;
    let scopes = data.claims.scope.as_ref().map(|value| value.to_vec()).unwrap_or_default();
    if !required_scopes.iter().all(|scope| scopes.contains(scope)) {
      return Err(ProviderError::InvalidToken);
    }
    crate::project_verified_access_token(&data)
  }
}
