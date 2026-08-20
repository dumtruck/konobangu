//! Stateless, attenuated capabilities for native browser resource requests.
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use libmacaroon::{Caveat, Format, Macaroon, MacaroonKey, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{AuthError, config::ResourceGrantsConfig};

const AUDIENCE: &str = "urn:konobangu:native-resources";
const PRIMARY_TTL_SECONDS: u64 = 3600;
pub const CHILD_TTL_SECONDS: u64 = 300;
const MAX_TOKEN_BYTES: usize = 4096;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PrimaryClaims {
  v: u8,
  kid: String,
  audience: String,
  subscriber_id: i32,
  scope: String,
  issued_at: u64,
  expires_at: u64,
  child_ttl_seconds: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResourceCaveat {
  subscriber_id: i32,
  path: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WindowCaveat {
  issued_at: u64,
  expires_at: u64,
}

/// Contains bearer material and deliberately has no Debug implementation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeResourceGrant {
  pub grant: String,
  pub issued_at: String,
  pub expires_at: String,
  pub child_ttl_seconds: u64,
}

pub struct NativeResourceGrants {
  key: MacaroonKey,
  kid: String,
}

impl std::fmt::Debug for NativeResourceGrants {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NativeResourceGrants").field("kid", &self.kid).finish_non_exhaustive()
  }
}

fn invalid() -> AuthError {
  AuthError::NativeResourceTokenInvalid
}

fn now() -> Result<u64, AuthError> {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|time| time.as_secs()).map_err(|_| invalid())
}

/// URI extraction has decoded the path once. Reject ambiguous storage paths
/// on every platform rather than letting platform-specific Path rules decide.
pub fn canonical_image_path(path: &str) -> bool {
  path.starts_with("image/")
    && path.split('/').count() >= 3
    && !path.contains(['\\', '\0', ':', '%'])
    && path.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

pub fn canonical_feed_path(path: &str) -> bool {
  path
    .strip_prefix("feeds/rss/")
    .is_some_and(|token| !token.is_empty() && token.len() <= 128 && token.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
}

impl NativeResourceGrants {
  pub fn new(config: &ResourceGrantsConfig) -> Result<Self, AuthError> {
    let key = match &config.signing_key {
      Some(encoded) => {
        let bytes: [u8; 32] = URL_SAFE_NO_PAD
          .decode(encoded)
          .ok()
          .and_then(|bytes| bytes.try_into().ok())
          .ok_or_else(|| AuthError::Configuration {
            message: "auth.resource_grants.signing_key must be 32 base64url-no-padding bytes".into(),
          })?;
        MacaroonKey::from(bytes)
      }
      None => MacaroonKey::generate_random().map_err(|_| AuthError::Configuration {
        message: "native resource signing entropy unavailable".into(),
      })?,
    };
    let bytes: &[u8] = key.as_ref();
    let kid = format!("{:x}", Sha256::digest(bytes))[..32].to_owned();
    Ok(Self { key, kid })
  }

  pub fn issue(&self, subscriber_id: i32) -> Result<NativeResourceGrant, AuthError> {
    self.issue_at(subscriber_id, now()?)
  }

  fn issue_at(&self, subscriber_id: i32, issued_at: u64) -> Result<NativeResourceGrant, AuthError> {
    if subscriber_id <= 0 {
      return Err(invalid());
    }
    let expires_at = issued_at.checked_add(PRIMARY_TTL_SECONDS).ok_or_else(invalid)?;
    let claims = PrimaryClaims {
      v: 1,
      kid: self.kid.clone(),
      audience: AUDIENCE.into(),
      subscriber_id,
      scope: "native-resource:read".into(),
      issued_at,
      expires_at,
      child_ttl_seconds: CHILD_TTL_SECONDS,
    };
    let identifier = serde_json::to_vec(&claims).map_err(|_| invalid())?;
    let grant = Macaroon::create(None, &self.key, identifier)
      .and_then(|macaroon| macaroon.serialize(Format::V2JSON))
      .map_err(|_| invalid())?;
    let stamp = |seconds| {
      chrono::DateTime::from_timestamp(i64::try_from(seconds).map_err(|_| invalid())?, 0)
        .map(|date| date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .ok_or_else(invalid)
    };
    Ok(NativeResourceGrant {
      grant,
      issued_at: stamp(issued_at)?,
      expires_at: stamp(expires_at)?,
      child_ttl_seconds: CHILD_TTL_SECONDS,
    })
  }

  pub fn verify(&self, encoded: &str, subscriber_id: i32, path: &str, method: &http::Method) -> Result<(), AuthError> {
    self.verify_at(encoded, subscriber_id, path, method, now()?)
  }

  fn verify_at(&self, encoded: &str, subscriber_id: i32, path: &str, method: &http::Method, now: u64) -> Result<(), AuthError> {
    if !canonical_image_path(path) {
      return Err(invalid());
    }
    self.verify_resource_at(encoded, Some(subscriber_id), path, method, now).map(|_| ())
  }

  pub fn verify_resource(&self, encoded: &str, expected_owner: Option<i32>, path: &str, method: &http::Method) -> Result<i32, AuthError> {
    self.verify_resource_at(encoded, expected_owner, path, method, now()?)
  }

  fn verify_resource_at(&self, encoded: &str, expected_owner: Option<i32>, path: &str, method: &http::Method, now: u64) -> Result<i32, AuthError> {
    if encoded.len() > MAX_TOKEN_BYTES
      || !matches!(*method, http::Method::GET | http::Method::HEAD)
      || !(canonical_image_path(path) || canonical_feed_path(path))
    {
      return Err(invalid());
    }
    let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| invalid())?;
    // Only the agreed JSON format is accepted, never auto-detected V1/V2
    // binary.
    if bytes.first() != Some(&b'{') {
      return Err(invalid());
    }
    let macaroon = Macaroon::deserialize(bytes).map_err(|_| invalid())?;
    if macaroon.location().is_some() || macaroon.caveats().len() != 2 {
      return Err(invalid());
    }
    let claims: PrimaryClaims = serde_json::from_slice(macaroon.identifier()).map_err(|_| invalid())?;
    if claims.v != 1
      || claims.kid != self.kid
      || claims.audience != AUDIENCE
      || claims.scope != "native-resource:read"
      || claims.subscriber_id <= 0
      || expected_owner.is_some_and(|owner| owner != claims.subscriber_id)
      || claims.child_ttl_seconds != CHILD_TTL_SECONDS
      || claims.expires_at.checked_sub(claims.issued_at) != Some(PRIMARY_TTL_SECONDS)
      || now < claims.issued_at
      || now >= claims.expires_at
    {
      return Err(invalid());
    }
    let predicate = |index| match &macaroon.caveats()[index] {
      Caveat::FirstParty(caveat) => Ok(caveat.predicate()),
      Caveat::ThirdParty(_) => Err(invalid()),
    };
    let resource_bytes = predicate(0)?;
    let window_bytes = predicate(1)?;
    let resource: ResourceCaveat = serde_json::from_slice(resource_bytes.strip_prefix(b"resource = ").ok_or_else(invalid)?).map_err(|_| invalid())?;
    let window: WindowCaveat = serde_json::from_slice(window_bytes.strip_prefix(b"window = ").ok_or_else(invalid)?).map_err(|_| invalid())?;
    if resource.subscriber_id != claims.subscriber_id
      || resource.path != path
      || window.issued_at < claims.issued_at
      || window.expires_at > claims.expires_at
      || window
        .expires_at
        .checked_sub(window.issued_at)
        .is_none_or(|ttl| ttl == 0 || ttl > CHILD_TTL_SECONDS)
      || now < window.issued_at
      || now >= window.expires_at
    {
      return Err(invalid());
    }
    let mut verifier = Verifier::default();
    verifier.satisfy_exact(resource_bytes);
    verifier.satisfy_exact(window_bytes);
    verifier.verify(&macaroon, &self.key, &[]).map_err(|_| invalid())?;
    Ok(claims.subscriber_id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn service() -> NativeResourceGrants {
    NativeResourceGrants::new(&ResourceGrantsConfig {
      signing_key: Some(URL_SAFE_NO_PAD.encode([7u8; 32])),
    })
    .unwrap()
  }

  fn child(grant: &str, owner: i32, path: &str, start: u64, end: u64) -> String {
    let mut macaroon = Macaroon::deserialize(grant).unwrap();
    macaroon
      .add_first_party_caveat(format!(
        "resource = {}",
        serde_json::to_string(&ResourceCaveat {
          subscriber_id: owner,
          path: path.into()
        })
        .unwrap()
      ))
      .unwrap();
    macaroon
      .add_first_party_caveat(format!(
        "window = {}",
        serde_json::to_string(&WindowCaveat {
          issued_at: start,
          expires_at: end
        })
        .unwrap()
      ))
      .unwrap();
    URL_SAFE_NO_PAD.encode(macaroon.serialize(Format::V2JSON).unwrap())
  }

  #[test]
  fn attenuated_grants_validate_only_one_image_with_a_bounded_window() {
    let service = service();
    let grant = service.issue_at(1001, 1000).unwrap();
    let token = child(&grant.grant, 1001, "image/posters/cover.png", 1000, 1300);
    for method in [http::Method::GET, http::Method::HEAD] {
      assert!(service.verify_at(&token, 1001, "image/posters/cover.png", &method, 1299).is_ok());
    }
    for (owner, path, method, time) in [
      (1002, "image/posters/cover.png", http::Method::GET, 1001),
      (1001, "image/posters/other.png", http::Method::GET, 1001),
      (1001, "image/posters/cover.png", http::Method::POST, 1001),
      (1001, "image/posters/cover.png", http::Method::GET, 999),
      (1001, "image/posters/cover.png", http::Method::GET, 1300),
    ] {
      assert!(service.verify_at(&token, owner, path, &method, time).is_err());
    }
    for (start, end) in [(1000, 1301), (999, 1200), (4500, 4700), (1000, 1000)] {
      let token = child(&grant.grant, 1001, "image/posters/cover.png", start, end);
      assert!(service.verify_at(&token, 1001, "image/posters/cover.png", &http::Method::GET, 1001).is_err());
    }
    assert!(
      service
        .verify_at(&URL_SAFE_NO_PAD.encode(&grant.grant), 1001, "image/posters/cover.png", &http::Method::GET, 1001)
        .is_err()
    );
    let mut json: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&token).unwrap()).unwrap();
    json["s64"] = serde_json::Value::String(URL_SAFE_NO_PAD.encode([0u8; 32]));
    assert!(
      service
        .verify_at(
          &URL_SAFE_NO_PAD.encode(json.to_string()),
          1001,
          "image/posters/cover.png",
          &http::Method::GET,
          1001
        )
        .is_err()
    );
    let mut macaroon = Macaroon::deserialize(&grant.grant).unwrap();
    macaroon.add_first_party_caveat("resource = {}").unwrap();
    macaroon.add_first_party_caveat("window = {}").unwrap();
    macaroon.add_first_party_caveat("extra = true").unwrap();
    assert!(
      service
        .verify_at(
          &URL_SAFE_NO_PAD.encode(macaroon.serialize(Format::V2JSON).unwrap()),
          1001,
          "image/posters/cover.png",
          &http::Method::GET,
          1001
        )
        .is_err()
    );
  }

  #[test]
  fn feed_capabilities_bind_one_locator_and_return_only_the_verified_owner() {
    let service = service();
    let grant = service.issue_at(1001, 1000).unwrap();
    let path = "feeds/rss/fixture-feed-A";
    let token = child(&grant.grant, 1001, path, 1000, 1300);
    assert_eq!(service.verify_resource_at(&token, None, path, &http::Method::GET, 1001).unwrap(), 1001);
    for (path, method, time) in [
      ("feeds/rss/fixture-feed-B", http::Method::GET, 1001),
      ("image/posters/cover.png", http::Method::GET, 1001),
      (path, http::Method::POST, 1001),
      (path, http::Method::GET, 1300),
      ("feeds/rss/a/b", http::Method::GET, 1001),
      ("feeds/rss/%2e", http::Method::GET, 1001),
      ("auth/user-info", http::Method::GET, 1001),
    ] {
      assert!(service.verify_resource_at(&token, None, path, &method, time).is_err());
    }
    let forged_owner = child(&grant.grant, 1002, path, 1000, 1300);
    assert!(service.verify_resource_at(&forged_owner, None, path, &http::Method::GET, 1001).is_err());
    assert!(service.verify_at(&token, 1001, path, &http::Method::GET, 1001).is_err());
  }

  #[test]
  fn unknown_duplicate_and_third_party_caveats_cannot_widen_a_primary() {
    let service = service();
    let grant = service.issue_at(1001, 1000).unwrap();
    let resource = r#"resource = {"subscriberId":1001,"path":"image/posters/cover.png"}"#;
    let window = r#"window = {"issuedAt":1000,"expiresAt":1300}"#;
    for predicates in [[window, resource], [resource, resource], [resource, "unknown = true"]] {
      let mut macaroon = Macaroon::deserialize(&grant.grant).unwrap();
      for predicate in predicates {
        macaroon.add_first_party_caveat(predicate).unwrap();
      }
      let token = URL_SAFE_NO_PAD.encode(macaroon.serialize(Format::V2JSON).unwrap());
      assert!(service.verify_at(&token, 1001, "image/posters/cover.png", &http::Method::GET, 1001).is_err());
    }
    let mut macaroon = Macaroon::deserialize(&grant.grant).unwrap();
    macaroon.add_first_party_caveat(resource).unwrap();
    macaroon
      .add_third_party_caveat("https://discharge.example", &MacaroonKey::from([8u8; 32]), window)
      .unwrap();
    let token = URL_SAFE_NO_PAD.encode(macaroon.serialize(Format::V2JSON).unwrap());
    assert!(service.verify_at(&token, 1001, "image/posters/cover.png", &http::Method::GET, 1001).is_err());
    let foreign = NativeResourceGrants::new(&ResourceGrantsConfig {
      signing_key: Some(URL_SAFE_NO_PAD.encode([9u8; 32])),
    })
    .unwrap();
    let valid = child(&grant.grant, 1001, "image/posters/cover.png", 1000, 1300);
    assert!(foreign.verify_at(&valid, 1001, "image/posters/cover.png", &http::Method::GET, 1001).is_err());
    assert!(
      service
        .verify_at(&"A".repeat(MAX_TOKEN_BYTES + 1), 1001, "image/posters/cover.png", &http::Method::GET, 1001)
        .is_err()
    );
  }

  #[test]
  fn cross_language_hmac_vector_matches_the_standard_library() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("../../tests/fixtures/native-resource-grants.json")).unwrap();
    let service = service();
    let grant = service.issue_at(1001, 1800000000).unwrap();
    assert_eq!(grant.grant, fixture["grant"]);
    assert!(
      service
        .verify_at(
          fixture["resourceToken"].as_str().unwrap(),
          1001,
          "image/posters/cover.png",
          &http::Method::GET,
          1800000001
        )
        .is_ok()
    );
    let generated = child(&grant.grant, 1001, "image/posters/cover.png", 1800000000, 1800000300);
    let generated = Macaroon::deserialize(URL_SAFE_NO_PAD.decode(generated).unwrap()).unwrap();
    let expected = Macaroon::deserialize(fixture["child"].as_str().unwrap()).unwrap();
    assert_eq!(generated, expected);
  }

  #[test]
  fn image_paths_and_signing_configuration_reject_ambiguous_inputs() {
    for path in [
      "image//cover.png",
      "image/a/../cover.png",
      "/image/a/cover.png",
      "image/a\\cover.png",
      "image/a/%2e%2e",
      "image/a/C:cover.png",
      "image/a/\0",
      "file/a/cover.png",
    ] {
      assert!(!canonical_image_path(path), "{path}");
    }
    for key in ["", "c2hvcnQ", "!!!!", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="] {
      assert!(NativeResourceGrants::new(&ResourceGrantsConfig { signing_key: Some(key.into()) }).is_err());
    }
    let a = NativeResourceGrants::new(&ResourceGrantsConfig::default()).unwrap();
    let b = NativeResourceGrants::new(&ResourceGrantsConfig::default()).unwrap();
    assert_ne!(a.kid, b.kid);
    let debug = format!(
      "{:?}",
      ResourceGrantsConfig {
        signing_key: Some("secret".into())
      }
    );
    assert!(!debug.contains("secret"));
  }
}
