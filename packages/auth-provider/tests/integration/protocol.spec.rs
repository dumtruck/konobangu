use std::{net::SocketAddr, sync::Arc};

use axum::{
  body::{Body, to_bytes},
  extract::ConnectInfo,
  http::{Request, StatusCode, header},
  response::Response,
};
use base64::{
  Engine,
  engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use konobangu_auth_provider::{
  ClientConfig, CredentialsConfig, DatabaseConfig, ProviderConfig, ProviderRuntime, SigningConfig, TokenConfig, apply_migrations, grant_access,
};
use konobangu_database::PoolConfig;
use openidconnect::{
  ClientId, IssuerUrl, Nonce,
  core::{CoreIdToken, CoreIdTokenVerifier},
};
use rsa::{RsaPrivateKey, pkcs8::EncodePrivateKey};
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;

struct Fixture {
  _postgres: ContainerAsync<Postgres>,
  owner: DatabaseConnection,
  runtime: Arc<ProviderRuntime>,
  config: ProviderConfig,
}
impl Fixture {
  async fn new() -> Self {
    let postgres = Postgres::default().with_tag("18-alpine").start().await.unwrap();
    let url = format!(
      "postgres://postgres:postgres@{}:{}/postgres",
      postgres.get_host().await.unwrap(),
      postgres.get_host_port_ipv4(5432).await.unwrap()
    );
    let pool = PoolConfig::default();
    let owner = konobangu_database::connect(&url, &pool, None).await.unwrap();
    let role = konobangu_database::roles::AUTH_PROVIDER_ACCESS_ROLE;
    owner
      .execute_unprepared(&format!(
        "DO $$ BEGIN CREATE ROLE {role} NOLOGIN NOINHERIT; EXCEPTION WHEN duplicate_object THEN NULL; END $$; GRANT {role} TO CURRENT_USER WITH INHERIT \
         FALSE, SET TRUE"
      ))
      .await
      .unwrap();
    apply_migrations(&owner).await.unwrap();
    grant_access(&owner).await.unwrap();
    let key = RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048)
      .unwrap()
      .to_pkcs8_pem(Default::default())
      .unwrap()
      .to_string();
    let config = ProviderConfig {
      issuer: "http://127.0.0.1:5071/auth-provider".into(),
      database: DatabaseConfig { url },
      signing: SigningConfig { private_key_pem: key },
      credentials: CredentialsConfig {
        password: "test-secret".into(),
        ..Default::default()
      },
      client: ClientConfig {
        redirect_uris: vec![
          "http://192.168.1.20:5000/auth/callback".into(),
          "https://another-ui.example/auth/callback".into(),
        ],
        ..Default::default()
      },
      tokens: TokenConfig {
        authorization_code_ttl_seconds: 2,
        ..Default::default()
      },
      ..Default::default()
    };
    let runtime = ProviderRuntime::new(config.clone(), pool).await.unwrap();
    Self {
      _postgres: postgres,
      owner,
      runtime,
      config,
    }
  }
  async fn request(&self, method: &str, path: &str, authorization: Option<&str>, body: Option<&str>) -> Response {
    let mut request = Request::builder()
      .method(method)
      .uri(path)
      .extension(ConnectInfo("127.0.0.1:12345".parse::<SocketAddr>().unwrap()));
    if let Some(auth) = authorization {
      request = request.header(header::AUTHORIZATION, auth);
    }
    if body.is_some() {
      request = request.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    }
    self
      .runtime
      .clone()
      .router()
      .oneshot(request.body(Body::from(body.unwrap_or_default().to_owned())).unwrap())
      .await
      .unwrap()
  }
  fn auth_query(&self, redirect: &str, challenge: &str) -> String {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query.extend_pairs([
      ("client_id", self.config.client.client_id.as_str()),
      ("redirect_uri", redirect),
      ("response_type", "code"),
      ("scope", "openid profile konobangu:api offline_access"),
      ("state", "state-value"),
      ("nonce", "nonce-value"),
      ("code_challenge_method", "S256"),
      ("code_challenge", challenge),
      ("response_mode", "query"),
    ]);
    query.finish()
  }
  async fn code(&self) -> String {
    self.code_with_scopes("openid profile konobangu:api offline_access").await
  }
  async fn code_with_scopes(&self, scopes: &str) -> String {
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier().as_bytes()));
    let response = self
      .request(
        "GET",
        &format!(
          "/authorize?{}",
          change_form(&self.auth_query(&self.config.client.redirect_uris[0], &challenge), "scope", Some(scopes))
        ),
        None,
        None,
      )
      .await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let login = url::Url::parse(response.headers()[header::LOCATION].to_str().unwrap()).unwrap();
    let path = format!("/login?{}", login.query().unwrap());
    let challenge = self.request("GET", &path, None, None).await;
    assert_eq!(challenge.status(), StatusCode::UNAUTHORIZED);
    assert!(challenge.headers()[header::WWW_AUTHENTICATE].to_str().unwrap().contains("Konobangu Provider"));
    let wrong = format!("Basic {}", STANDARD.encode("admin:wrong"));
    assert_eq!(self.request("GET", &path, Some(&wrong), None).await.status(), StatusCode::UNAUTHORIZED);
    let auth = format!("Basic {}", STANDARD.encode("admin:test-secret"));
    let response = self.request("GET", &path, Some(&auth), None).await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(self.request("GET", &path, Some(&auth), None).await.status(), StatusCode::BAD_REQUEST);
    let redirect = url::Url::parse(response.headers()[header::LOCATION].to_str().unwrap()).unwrap();
    let pairs: std::collections::HashMap<_, _> = redirect.query_pairs().collect();
    assert_eq!(pairs["state"], "state-value");
    pairs["code"].to_string()
  }
  fn exchange_body(&self, code: &str) -> String {
    url::form_urlencoded::Serializer::new(String::new())
      .extend_pairs([
        ("grant_type", "authorization_code"),
        ("client_id", &self.config.client.client_id),
        ("code", code),
        ("redirect_uri", &self.config.client.redirect_uris[0]),
        ("code_verifier", verifier()),
      ])
      .finish()
  }
  async fn tokens(&self) -> Value {
    let code = self.code().await;
    let response = self.request("POST", "/token", None, Some(&self.exchange_body(&code))).await;
    assert_eq!(response.status(), StatusCode::OK, "{}", body(response).await);
    body(response).await
  }
  async fn refresh(&self, token: &str) -> Response {
    let form = url::form_urlencoded::Serializer::new(String::new())
      .extend_pairs([
        ("grant_type", "refresh_token"),
        ("client_id", self.config.client.client_id.as_str()),
        ("refresh_token", token),
      ])
      .finish();
    self.request("POST", "/token", None, Some(&form)).await
  }
}
fn verifier() -> &'static str {
  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~"
}
async fn body(response: Response) -> Value {
  serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
}

fn change_form(form: &str, key: &str, value: Option<&str>) -> String {
  let pairs: Vec<(String, String)> = url::form_urlencoded::parse(form.as_bytes())
    .filter(|(name, _)| name != key)
    .map(|(name, value)| (name.into_owned(), value.into_owned()))
    .collect();
  let mut serializer = url::form_urlencoded::Serializer::new(String::new());
  serializer.extend_pairs(pairs);
  if let Some(value) = value {
    serializer.append_pair(key, value);
  }
  serializer.finish()
}

#[tokio::test]
async fn postgres_oidc_protocol_and_atomic_grants() {
  let fixture = Fixture::new().await;
  let discovery = body(fixture.request("GET", "/.well-known/openid-configuration", None, None).await).await;
  assert_eq!(discovery["code_challenge_methods_supported"], json!(["S256"]));
  let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier().as_bytes()));
  for query in [
    fixture.auth_query("https://evil.example/auth/callback", &challenge),
    fixture.auth_query(&fixture.config.client.redirect_uris[0], &challenge).replace("S256", "plain"),
    format!("{}&client_id=another", fixture.auth_query(&fixture.config.client.redirect_uris[0], &challenge)),
  ] {
    let response = fixture.request("GET", &format!("/authorize?{query}"), None, None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(!response.headers().contains_key(header::LOCATION));
    assert!(!response.headers().contains_key(header::WWW_AUTHENTICATE));
  }
  let registered = &fixture.config.client.redirect_uris[0];
  for query in [
    change_form(&fixture.auth_query(registered, &challenge), "code_challenge", None),
    change_form(&fixture.auth_query(registered, &challenge), "response_type", Some("token")),
    change_form(&fixture.auth_query(registered, &challenge), "scope", Some("openid forbidden")),
    change_form(&fixture.auth_query(registered, &challenge), "prompt", Some("none")),
  ] {
    assert_eq!(
      fixture.request("GET", &format!("/authorize?{query}"), None, None).await.status(),
      StatusCode::BAD_REQUEST
    );
  }
  for (key, value) in [
    ("code_verifier", None),
    ("code_verifier", Some("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789xx")),
    ("redirect_uri", Some("https://another-ui.example/auth/callback")),
    ("client_id", Some("wrong-client")),
    ("client_id", None),
  ] {
    let code = fixture.code().await;
    let form = change_form(&fixture.exchange_body(&code), key, value);
    let response = fixture.request("POST", "/token", None, Some(&form)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{key}");
    let expected = if value.is_none() {
      "invalid_request"
    } else if key == "client_id" {
      "invalid_client"
    } else {
      "invalid_grant"
    };
    assert_eq!(body(response).await["error"], expected, "{key}");
  }
  let code = fixture.code().await;
  let form = fixture.exchange_body(&code);
  assert_eq!(
    fixture
      .request("POST", "/token", None, Some(&format!("{form}&code_verifier={}", verifier())))
      .await
      .status(),
    StatusCode::BAD_REQUEST
  );
  let basic = format!("Basic {}", STANDARD.encode("admin:test-secret"));
  assert_eq!(
    fixture.request("POST", "/token", Some(&basic), Some(&form)).await.status(),
    StatusCode::BAD_REQUEST
  );
  let denied = fixture.request("GET", "/userinfo", Some(&basic), None).await;
  assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
  assert_eq!(denied.headers()[header::WWW_AUTHENTICATE], "Bearer error=\"invalid_token\"");
  let expired = fixture.code().await;
  tokio::time::sleep(std::time::Duration::from_millis(2200)).await;
  let response = fixture.request("POST", "/token", None, Some(&fixture.exchange_body(&expired))).await;
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  assert_eq!(body(response).await["error"], "invalid_grant");
  let mut foreign_config = fixture.config.clone();
  foreign_config.issuer = "http://127.0.0.1:5072/auth-provider".into();
  let foreign = ProviderRuntime::new(foreign_config, PoolConfig::default()).await.unwrap();
  let code = fixture.code().await;
  let foreign_request = Request::post("/token")
    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
    .body(Body::from(fixture.exchange_body(&code)))
    .unwrap();
  assert_eq!(
    foreign.clone().router().oneshot(foreign_request).await.unwrap().status(),
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    fixture.request("POST", "/token", None, Some(&fixture.exchange_body(&code))).await.status(),
    StatusCode::OK
  );
  foreign.close().await.unwrap();
  let jwks = body(fixture.request("GET", "/jwks", None, None).await).await;
  let public_key = &jwks["keys"][0];
  assert!(public_key.get("n").is_some());
  for private in ["d", "p", "q", "dp", "dq", "qi"] {
    assert!(public_key.get(private).is_none());
  }
  let openid_code = fixture.code_with_scopes("openid").await;
  let openid_tokens = fixture.request("POST", "/token", None, Some(&fixture.exchange_body(&openid_code))).await;
  assert_eq!(openid_tokens.status(), StatusCode::OK);
  let openid_tokens = body(openid_tokens).await;
  assert!(openid_tokens.get("refresh_token").is_none());
  let openid_access = openid_tokens["access_token"].as_str().unwrap();
  assert!(fixture.runtime.verify_access_token(openid_access).is_err());
  assert_eq!(
    fixture
      .request("GET", "/userinfo", Some(&format!("Bearer {openid_access}")), None)
      .await
      .status(),
    StatusCode::OK
  );
  let tokens = fixture.tokens().await;
  let userinfo = body(
    fixture
      .request("GET", "/userinfo", Some(&format!("Bearer {}", tokens["access_token"].as_str().unwrap())), None)
      .await,
  )
  .await;
  assert_eq!(userinfo["sub"], "local-admin");
  let bearer = tokens["access_token"].as_str().unwrap();
  let principal = fixture.runtime.verify_access_token(bearer).unwrap();
  assert_eq!(principal.subject, "local-admin");
  let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(bearer.split('.').nth(1).unwrap()).unwrap()).unwrap();
  let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
  header.typ = Some("at+jwt".into());
  header.kid = Some(public_key["kid"].as_str().unwrap().to_owned());
  let key = jsonwebtoken::EncodingKey::from_rsa_pem(fixture.config.signing.private_key_pem.as_bytes()).unwrap();
  for (field, value) in [
    ("iss", json!("https://wrong.example/auth-provider")),
    ("aud", json!("wrong-api")),
    ("exp", json!(chrono::Utc::now().timestamp() - 120)),
    ("nbf", json!(chrono::Utc::now().timestamp() + 120)),
    ("scope", json!("openid")),
    ("sub", json!("")),
  ] {
    let mut invalid = claims.clone();
    invalid[field] = value;
    let token = jsonwebtoken::encode(&header, &invalid, &key).unwrap();
    assert!(fixture.runtime.verify_access_token(&token).is_err(), "{field}");
  }
  let id_token: CoreIdToken = tokens["id_token"].as_str().unwrap().parse().unwrap();
  let verifier = CoreIdTokenVerifier::new_public_client(
    ClientId::new(fixture.config.client.client_id.clone()),
    IssuerUrl::new(fixture.config.issuer.clone()).unwrap(),
    fixture.runtime.jwks(),
  );
  let claims = id_token.claims(&verifier, &Nonce::new("nonce-value".into())).unwrap();
  assert_eq!(claims.subject().as_str(), "local-admin");
  assert!(claims.access_token_hash().is_some());
  assert!(fixture.runtime.verify_access_token(tokens["id_token"].as_str().unwrap()).is_err());
  assert!(fixture.runtime.verify_access_token(tokens["refresh_token"].as_str().unwrap()).is_err());
  // A refresh outlives the short authorization-code window.
  tokio::time::sleep(std::time::Duration::from_millis(2200)).await;
  let refreshed = fixture.refresh(tokens["refresh_token"].as_str().unwrap()).await;
  assert_eq!(refreshed.status(), StatusCode::OK);
  let refreshed = body(refreshed).await;
  assert_eq!(
    fixture.refresh(tokens["refresh_token"].as_str().unwrap()).await.status(),
    StatusCode::BAD_REQUEST
  );
  assert_eq!(
    fixture.refresh(refreshed["refresh_token"].as_str().unwrap()).await.status(),
    StatusCode::BAD_REQUEST
  );
  let code = fixture.code().await;
  let form = fixture.exchange_body(&code);
  let (a, b) = tokio::join!(
    fixture.request("POST", "/token", None, Some(&form)),
    fixture.request("POST", "/token", None, Some(&form))
  );
  assert_eq!([a.status(), b.status()].into_iter().filter(|status| *status == StatusCode::OK).count(), 1);
  let tokens = fixture.tokens().await;
  let token = tokens["refresh_token"].as_str().unwrap();
  let (a, b) = tokio::join!(fixture.refresh(token), fixture.refresh(token));
  assert_eq!([a.status(), b.status()].into_iter().filter(|status| *status == StatusCode::OK).count(), 1);
  let winner = if a.status() == StatusCode::OK { body(a).await } else { body(b).await };
  assert_eq!(
    fixture.refresh(winner["refresh_token"].as_str().unwrap()).await.status(),
    StatusCode::BAD_REQUEST
  );
  // Once an ancestor is reused, no concurrent descendant may retain refresh
  // rights.
  for _ in 0..4 {
    let first = fixture.tokens().await;
    let old = first["refresh_token"].as_str().unwrap();
    let next = body(fixture.refresh(old).await).await;
    let (replay, descendant) = tokio::join!(fixture.refresh(old), fixture.refresh(next["refresh_token"].as_str().unwrap()));
    assert_eq!(replay.status(), StatusCode::BAD_REQUEST);
    if descendant.status() == StatusCode::OK {
      let last = body(descendant).await;
      assert_eq!(fixture.refresh(last["refresh_token"].as_str().unwrap()).await.status(), StatusCode::BAD_REQUEST);
    }
  }
  let tokens = fixture.tokens().await;
  let original = tokens["refresh_token"].as_str().unwrap();
  let form = url::form_urlencoded::Serializer::new(String::new())
    .extend_pairs([
      ("grant_type", "refresh_token"),
      ("client_id", fixture.config.client.client_id.as_str()),
      ("refresh_token", original),
      ("scope", "openid konobangu:api offline_access"),
    ])
    .finish();
  let narrowed = fixture.request("POST", "/token", None, Some(&form)).await;
  assert_eq!(narrowed.status(), StatusCode::OK);
  let narrowed = body(narrowed).await;
  assert!(!narrowed["scope"].as_str().unwrap().contains("profile"));
  let narrowed_token = narrowed["refresh_token"].as_str().unwrap();
  let expanded = change_form(
    &change_form(&form, "refresh_token", Some(narrowed_token)),
    "scope",
    Some("openid profile konobangu:api offline_access"),
  );
  let expanded = fixture.request("POST", "/token", None, Some(&expanded)).await;
  assert_eq!(expanded.status(), StatusCode::BAD_REQUEST);
  assert_eq!(body(expanded).await["error"], "invalid_scope");
  assert_eq!(fixture.refresh(narrowed_token).await.status(), StatusCode::OK);
  // Startup rejects permissions added to the provider capability.
  let role = konobangu_database::roles::AUTH_PROVIDER_ACCESS_ROLE;
  fixture
    .owner
    .execute_unprepared(&format!(
      "CREATE TABLE public.unrelated_business(id integer); GRANT SELECT ON public.unrelated_business TO {role}"
    ))
    .await
    .unwrap();
  assert!(ProviderRuntime::new(fixture.config.clone(), PoolConfig::default()).await.is_err());
  fixture
    .owner
    .execute_unprepared(&format!("REVOKE SELECT ON public.unrelated_business FROM {role}"))
    .await
    .unwrap();
  // Restart keeps keys and protocol grants, without HTTP self-discovery.
  let tokens = fixture.tokens().await;
  let previous = fixture.runtime.jwks();
  fixture.runtime.close().await.unwrap();
  let restarted = ProviderRuntime::new(fixture.config.clone(), PoolConfig::default()).await.unwrap();
  assert_eq!(serde_json::to_value(previous).unwrap(), serde_json::to_value(restarted.jwks()).unwrap());
  assert!(restarted.verify_access_token(tokens["access_token"].as_str().unwrap()).is_ok());
  let form = url::form_urlencoded::Serializer::new(String::new())
    .extend_pairs([
      ("grant_type", "refresh_token"),
      ("client_id", fixture.config.client.client_id.as_str()),
      ("refresh_token", tokens["refresh_token"].as_str().unwrap()),
    ])
    .finish();
  let response = restarted
    .clone()
    .router()
    .oneshot(
      Request::post("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  restarted.close().await.unwrap();
  let response = fixture.refresh(tokens["refresh_token"].as_str().unwrap()).await;
  assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
  assert_eq!(body(response).await["error"], "server_error");
  fixture.owner.close().await.unwrap();
}
