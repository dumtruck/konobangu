//! Browser entry selection is request-local and checked against the SDK's
//! redirect-target rules. API authority and forwarded headers never identify
//! a separately deployed WebUI.
use http::{HeaderMap, StatusCode};
use securitydept_utils::redirect::{RedirectTargetConfig, RedirectTargetRule, UriRedirectTargetResolver};
use url::Url;

/// The router and runtime configuration use the WebUI origin root.
/// Changing these paths requires updating both client and server routing.
pub const CALLBACK_PATH: &str = "/auth/callback";
pub const RUNTIME_CONFIG_PATH: &str = "/konobangu.config.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebuiOriginError(&'static str);

impl std::fmt::Display for WebuiOriginError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "auth.webui.allowed_origins {}", self.0)
  }
}

impl std::error::Error for WebuiOriginError {}

/// A validated origin selected for one request, never global deployment state.
#[derive(Debug, Clone)]
pub struct WebuiOrigin {
  origin: String,
}

impl WebuiOrigin {
  /// Accepts an explicitly allowed HTTP(S) origin root.
  /// A path, query, fragment or user information is rejected rather than
  /// silently trimmed, because the WebUI is served from this origin's root.
  pub fn parse(raw: &str) -> Result<Self, WebuiOriginError> {
    if raw.chars().any(char::is_whitespace) || raw.contains('\\') {
      return Err(WebuiOriginError("must not contain whitespace or backslashes"));
    }
    let url = Url::parse(raw).map_err(|_| WebuiOriginError("must be a valid absolute URL"))?;
    match url.scheme() {
      "https" | "http" => {}
      _ => return Err(WebuiOriginError("must use http(s)")),
    }
    if url.host_str().is_none_or(str::is_empty) {
      return Err(WebuiOriginError("must name a host"));
    }
    if !url.username().is_empty() || url.password().is_some() || has_user_information(raw) {
      return Err(WebuiOriginError("must not carry user information"));
    }
    if url.query().is_some() {
      return Err(WebuiOriginError("must not carry a query"));
    }
    if url.fragment().is_some() {
      return Err(WebuiOriginError("must not carry a fragment"));
    }
    if url.path() != "/" && !url.path().is_empty() {
      return Err(WebuiOriginError("must be an origin root without a path"));
    }
    let origin = url.origin().ascii_serialization();
    if origin == "null" {
      return Err(WebuiOriginError("must have a serializable origin"));
    }
    Ok(Self { origin })
  }

  /// The selected browser origin, including its protocol and port.
  pub fn origin(&self) -> &str {
    &self.origin
  }

  /// The callback for this allowlisted browser entry.
  pub fn callback_url(&self) -> String {
    format!("{}{CALLBACK_PATH}", self.origin)
  }

  /// Where the deployment publishes its public runtime configuration sidecar.
  pub fn runtime_config_url(&self) -> String {
    format!("{}{RUNTIME_CONFIG_PATH}", self.origin)
  }
}

fn has_user_information(raw: &str) -> bool {
  raw
    .split_once("://")
    .is_some_and(|(_, remainder)| remainder.split(['/', '?', '#']).next().is_some_and(|authority| authority.contains('@')))
}

/// One immutable allowlist shared by source checks, CORS and login targets.
#[derive(Debug, Clone)]
pub struct WebuiPolicy {
  resolver: UriRedirectTargetResolver,
}

impl WebuiPolicy {
  pub fn new(rules: &[RedirectTargetRule]) -> Result<Self, WebuiOriginError> {
    let mut normalized = Vec::with_capacity(rules.len());
    for rule in rules {
      normalized.push(match rule {
        RedirectTargetRule::Strict { value } => RedirectTargetRule::Strict {
          value: WebuiOrigin::parse(value)?.origin,
        },
        RedirectTargetRule::Regex { value } => RedirectTargetRule::Regex {
          // The SDK normally permits substring matching; origins must match
          // the entire rule, including protocol and port.
          value: regex::Regex::new(&format!("\\A(?:{})\\z", value.as_str())).map_err(|_| WebuiOriginError("contains an invalid origin pattern"))?,
        },
        RedirectTargetRule::All => return Err(WebuiOriginError("must not permit all origins")),
      });
    }
    let resolver = UriRedirectTargetResolver::from_config(RedirectTargetConfig::dynamic_targets(normalized))
      .map_err(|_| WebuiOriginError("must contain at least one valid origin rule"))?;
    Ok(Self { resolver })
  }

  pub fn allows_origin(&self, raw: &str) -> bool {
    self.resolve_origin(raw).is_ok()
  }

  fn resolve_origin(&self, raw: &str) -> Result<WebuiOrigin, StatusCode> {
    let origin = WebuiOrigin::parse(raw).map_err(|_| StatusCode::FORBIDDEN)?;
    if raw != origin.origin() {
      return Err(StatusCode::FORBIDDEN);
    }
    self
      .resolver
      .resolve_redirect_target(Some(origin.origin()))
      .map_err(|_| StatusCode::FORBIDDEN)?;
    Ok(origin)
  }

  /// A provided Origin is authoritative: invalid, opaque or duplicate values
  /// cannot be repaired by a Referer or by an API/forwarded host.
  pub fn optional_source(&self, headers: &HeaderMap) -> Result<Option<WebuiOrigin>, StatusCode> {
    if headers.contains_key(http::header::ORIGIN) {
      if headers.get_all(http::header::ORIGIN).iter().count() != 1 {
        return Err(StatusCode::FORBIDDEN);
      }
      let origin = headers[http::header::ORIGIN].to_str().map_err(|_| StatusCode::FORBIDDEN)?;
      return self.resolve_origin(origin).map(Some);
    }
    if headers.contains_key(http::header::REFERER) {
      if headers.get_all(http::header::REFERER).iter().count() != 1 {
        return Err(StatusCode::FORBIDDEN);
      }
      let referer = headers[http::header::REFERER].to_str().map_err(|_| StatusCode::FORBIDDEN)?;
      if referer.chars().any(char::is_whitespace) || referer.contains('\\') {
        return Err(StatusCode::FORBIDDEN);
      }
      let url = Url::parse(referer).map_err(|_| StatusCode::FORBIDDEN)?;
      if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || has_user_information(referer)
        || url.fragment().is_some()
      {
        return Err(StatusCode::FORBIDDEN);
      }
      return self.resolve_origin(&url.origin().ascii_serialization()).map(Some);
    }
    Ok(None)
  }

  pub fn resolve_source(&self, headers: &HeaderMap) -> Result<WebuiOrigin, StatusCode> {
    self.optional_source(headers)?.ok_or(StatusCode::FORBIDDEN)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn origin() -> WebuiOrigin {
    WebuiOrigin::parse("https://ui.konobangu.test/").unwrap()
  }

  #[test]
  fn accepts_an_https_origin_root_and_loopback_development() {
    assert_eq!(origin().origin(), "https://ui.konobangu.test");
    assert_eq!(WebuiOrigin::parse("https://ui.konobangu.test").unwrap().origin(), "https://ui.konobangu.test");
    assert_eq!(WebuiOrigin::parse("http://localhost:5000/").unwrap().origin(), "http://localhost:5000");
    assert_eq!(WebuiOrigin::parse("http://127.0.0.1:5000").unwrap().origin(), "http://127.0.0.1:5000");
  }

  #[test]
  fn rejects_anything_other_than_a_bare_origin() {
    for raw in [
      "ftp://ui.konobangu.test/",
      "https://user:pw@ui.konobangu.test/",
      "https://ui.konobangu.test/app",
      "https://ui.konobangu.test/?tenant=a",
      "https://ui.konobangu.test/#x",
      "ui.konobangu.test",
      "",
    ] {
      assert!(WebuiOrigin::parse(raw).is_err(), "{raw} must be rejected");
    }
  }

  #[test]
  fn derives_fixed_targets_from_configuration_only() {
    let webui = origin();
    assert_eq!(webui.callback_url(), "https://ui.konobangu.test/auth/callback");
    assert_eq!(webui.runtime_config_url(), "https://ui.konobangu.test/konobangu.config.json");
  }

  fn policy() -> WebuiPolicy {
    WebuiPolicy::new(&[
      RedirectTargetRule::Strict {
        value: "https://ui.konobangu.test/".into(),
      },
      RedirectTargetRule::Strict {
        value: "http://192.168.1.20:5000".into(),
      },
      RedirectTargetRule::Regex {
        value: regex::Regex::new(r"https://app-[a-z]+\.region\.example").unwrap(),
      },
    ])
    .unwrap()
  }

  #[test]
  fn policy_selects_multiple_explicit_origins_and_matches_whole_patterns() {
    let policy = policy();
    for origin in ["https://ui.konobangu.test", "http://192.168.1.20:5000", "https://app-east.region.example"] {
      assert!(policy.allows_origin(origin), "{origin}");
    }
    for origin in [
      "http://ui.konobangu.test",
      "http://192.168.1.20:5001",
      "https://app-east.region.example.attacker.test",
      "https://evil.test/path/https://app-east.region.example",
      "null",
    ] {
      assert!(!policy.allows_origin(origin), "{origin}");
    }
    assert!(WebuiPolicy::new(&[]).is_err());
    assert!(WebuiPolicy::new(&[RedirectTargetRule::All]).is_err());
  }

  #[test]
  fn origin_is_authoritative_and_referer_is_only_a_missing_origin_fallback() {
    let policy = policy();
    let mut headers = HeaderMap::new();
    headers.insert("referer", "http://192.168.1.20:5000/subscriptions?view=all".parse().unwrap());
    assert_eq!(policy.resolve_source(&headers).unwrap().origin(), "http://192.168.1.20:5000");
    headers.insert("origin", "https://ui.konobangu.test".parse().unwrap());
    assert_eq!(policy.resolve_source(&headers).unwrap().origin(), "https://ui.konobangu.test");
    for invalid in ["null", "https://attacker.example", "https://ui.konobangu.test/path"] {
      headers.insert("origin", invalid.parse().unwrap());
      assert!(policy.resolve_source(&headers).is_err());
    }
    headers.remove("origin");
    headers.insert("referer", "https://user@ui.konobangu.test/path".parse().unwrap());
    assert!(policy.resolve_source(&headers).is_err());
    headers.clear();
    headers.append("origin", "https://ui.konobangu.test".parse().unwrap());
    headers.append("origin", "https://ui.konobangu.test".parse().unwrap());
    assert!(policy.resolve_source(&headers).is_err());
    headers.clear();
    headers.append("referer", "https://ui.konobangu.test/a".parse().unwrap());
    headers.append("referer", "https://ui.konobangu.test/b".parse().unwrap());
    assert!(policy.resolve_source(&headers).is_err());
    headers.clear();
    headers.insert("x-forwarded-host", "ui.konobangu.test".parse().unwrap());
    headers.insert("host", "ui.konobangu.test".parse().unwrap());
    assert!(policy.resolve_source(&headers).is_err());
    assert!(policy.optional_source(&headers).unwrap().is_none());
  }
}
