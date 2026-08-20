//! Resource-server verification against a real signed-token fixture.
//!
//! The SDK owns discovery, JWKS and the standard claim checks; this proves the
//! deployment's full policy, including the two checks this host adds on top.
use recorder::auth::{
  config::{OidcAuthConfig, OidcBrowserConfig, OidcResourceConfig},
  resource::ResourceVerifier,
};

fn config(issuer: &str) -> OidcAuthConfig {
  OidcAuthConfig {
    issuer: issuer.to_owned(),
    trusted_ca_file: None,
    browser: OidcBrowserConfig {
      client_id: "konobangu-webui".into(),
      scopes: vec!["openid".into(), "profile".into(), "konobangu:api".into()],
    },
    resource: OidcResourceConfig::default(),
  }
}

#[tokio::test]
async fn resource_server_accepts_only_access_tokens_for_this_api() {
  for trailing_slash in [false, true] {
    let mut server = mockito::Server::new_async().await;
    let issuer = format!("{}{}", server.url(), if trailing_slash { "/" } else { "" });
    let generator = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../webui/tests/support/access-token-vectors.mts");
    let output = std::process::Command::new("node")
      .arg(generator)
      .arg(&issuer)
      .output()
      .expect("mise Node and installed jose are required");
    assert!(
      output.status.success(),
      "jose fixture generation failed: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let generated: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let metadata = serde_json::json!({
      "issuer": issuer,
      "authorization_endpoint": format!("{}/authorize", server.url()),
      "token_endpoint": format!("{}/token", server.url()),
      "jwks_uri": format!("{}/jwks", server.url()),
      "response_types_supported": ["code"],
      "subject_types_supported": ["public"],
      "id_token_signing_alg_values_supported": ["RS256"],
      "token_endpoint_auth_methods_supported": ["none"],
      "code_challenge_methods_supported": ["S256"]
    });
    let _discovery = server
      .mock("GET", "/.well-known/openid-configuration")
      .with_header("content-type", "application/json")
      .with_body(metadata.to_string())
      .expect_at_least(1)
      .create_async()
      .await;
    let _jwks = server
      .mock("GET", "/jwks")
      .with_header("content-type", "application/json")
      .with_body(generated["jwks"].to_string())
      .create_async()
      .await;

    let verifier = ResourceVerifier::new(&config(&issuer)).await.expect("resource verifier");
    let token = |name: &str| generated["tokens"][name].as_str().expect("fixture token").to_owned();

    let principal = verifier.authenticate(&token("valid")).await.expect("a valid access token is accepted");
    assert_eq!(principal.subject, "resource-account");
    assert_eq!(principal.issuer, issuer);
    assert!(principal.audiences.contains(&"urn:konobangu:api".to_owned()));
    assert!(principal.scopes.contains(&"konobangu:api".to_owned()));
    assert_eq!(principal.client_id.as_deref(), Some("konobangu-webui"));

    // Standard claim and signature checks come from the SDK entry point.
    for scenario in [
      "issuer",
      "issuerSlashMismatch",
      "audience",
      "expired",
      "notBefore",
      "signature",
      "scope",
      "emptySubject",
    ] {
      assert!(verifier.authenticate(&token(scenario)).await.is_err(), "{scenario} must be rejected");
    }
    // These two are this host's additions: an ID token and an untyped token are
    // validly signed for the same issuer and audience, and a symmetric
    // signature must never be accepted against published public keys.
    for scenario in ["idToken", "missingType", "symmetric"] {
      assert!(verifier.authenticate(&token(scenario)).await.is_err(), "{scenario} must be rejected");
    }
    // A Basic credential must not reach the resource path at all.
    assert!(verifier.authenticate("").await.is_err());
  }
}

/// Exercises the native-token and bearer paths against the production router,
/// including authorization before storage metadata and conditional replies.
#[tokio::test]
async fn native_resource_capabilities_are_oidc_issued_and_route_bound() {
  use std::sync::Arc;

  use axum::Router;
  use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
  use fetch::reqwest;
  use futures::FutureExt;
  use libmacaroon::{Format, Macaroon};
  use recorder::{
    app::AppContextTrait,
    auth::AuthService,
    media::{MediaConfig, MediaService},
    storage::{StorageConfig, StorageService},
    test_utils::app::TestingAppContext,
    web::controller::{ControllerTrait, auth, feeds, r#static},
  };
  let fixture = super::support::RlsFixture::new().await;
  let mut issuer = mockito::Server::new_async().await;
  let url = issuer.url();
  let generator = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../webui/tests/support/access-token-vectors.mts");
  let output = std::process::Command::new("node").arg(generator).arg(&url).output().unwrap();
  assert!(output.status.success());
  let generated: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  let _discovery = issuer.mock("GET", "/.well-known/openid-configuration")
    .with_header("content-type","application/json")
    .with_body(serde_json::json!({"issuer":url,"authorization_endpoint":format!("{url}/authorize"),"token_endpoint":format!("{url}/token"),"jwks_uri":format!("{url}/jwks"),"response_types_supported":["code"],"subject_types_supported":["public"],"id_token_signing_alg_values_supported":["RS256"]}).to_string())
    .expect_at_least(1).create_async().await;
  let _jwks = issuer
    .mock("GET", "/jwks")
    .with_header("content-type", "application/json")
    .with_body(generated["jwks"].to_string())
    .expect_at_least(1)
    .create_async()
    .await;
  let config: recorder::auth::AuthConfig = serde_json::from_value(serde_json::json!({
    "webui":{"allowed_origins":[{"type":"strict","value":"https://ui.example"}]},
    "identity":{"database_url":fixture.identity_uri},
    "provider":{"type":"oidc","issuer":url,"browser":{"client_id":"konobangu-webui"}},
    "resource_grants":{"signing_key":URL_SAFE_NO_PAD.encode([7u8;32])}
  }))
  .unwrap();
  let application_config = fixture.runtime_config(config.clone());
  let directory = tempfile::tempdir().unwrap();
  let ctx: Arc<dyn AppContextTrait> = Arc::new(
    TestingAppContext::builder()
      .db(
        recorder::database::DatabaseService::from_config(application_config.database.clone())
          .await
          .unwrap(),
      )
      .config(application_config)
      .auth(AuthService::from_conf(config).await.unwrap())
      .storage(
        StorageService::from_config(StorageConfig {
          data_dir: directory.path().to_string_lossy().into_owned(),
        })
        .await
        .unwrap(),
      )
      .media(MediaService::from_config(MediaConfig::default()).await.unwrap())
      .build(),
  );
  let mut router = Router::new();
  for controller in [
    auth::create(ctx.clone()).await.unwrap(),
    r#static::create(ctx.clone()).await.unwrap(),
    feeds::create(ctx.clone()).await.unwrap(),
  ] {
    router = controller.apply_to(router);
  }
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let base = format!("http://{}", listener.local_addr().unwrap());
  let (shutdown, stopped) = tokio::sync::oneshot::channel();
  let server_context = ctx.clone();
  let served = tokio::spawn(async move {
    axum::serve(listener, router.with_state(server_context))
      .with_graceful_shutdown(async {
        let _ = stopped.await;
      })
      .await
  });
  let client = reqwest::Client::new();
  let outcome = std::panic::AssertUnwindSafe(async {
    let access = generated["tokens"]["valid"].as_str().unwrap();
    let issue = format!("{base}/api/auth/resource-grants");
    assert_eq!(client.post(&issue).json(&serde_json::json!({})).send().await.unwrap().status(), 401);
    assert_eq!(
      client
        .post(&issue)
        .basic_auth("fixture-admin", Some("fixture-password"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .status(),
      401
    );
    let response = client.post(&issue).bearer_auth(access).json(&serde_json::json!({})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(
      client
        .post(&issue)
        .bearer_auth(access)
        .json(&serde_json::json!({"subscriberId":1002}))
        .send()
        .await
        .unwrap()
        .status(),
      422
    );
    let grant: serde_json::Value = response.json().await.unwrap();
    assert_eq!(grant["childTtlSeconds"], 300);
    let primary = Macaroon::deserialize(grant["grant"].as_str().unwrap()).unwrap();
    let claims: serde_json::Value = serde_json::from_slice(primary.identifier()).unwrap();
    let owner = claims["subscriberId"].as_i64().unwrap();
    let issued = claims["issuedAt"].as_u64().unwrap();
    let derive = |path: &str, start: u64, end: u64| {
      let mut child = primary.clone();
      child
        .add_first_party_caveat(format!("resource = {}", serde_json::json!({"subscriberId":owner,"path":path})))
        .unwrap();
      child
        .add_first_party_caveat(format!("window = {}", serde_json::json!({"issuedAt":start,"expiresAt":end})))
        .unwrap();
      URL_SAFE_NO_PAD.encode(child.serialize(Format::V2JSON).unwrap())
    };
    fixture.seed_rss_feed(owner.try_into().unwrap(), "native-feed-A").await;
    fixture.seed_rss_feed(1002, "native-feed-B").await;
    let feed_path = "feeds/rss/native-feed-A";
    let feed_token = derive(feed_path, issued, issued + 300);
    let feed_url = format!("{base}/api/{feed_path}?resource_token={feed_token}");
    let feed = client.get(&feed_url).send().await.unwrap();
    assert_eq!(feed.status(), 200);
    assert_eq!(feed.headers()["cache-control"], "no-store");
    assert_eq!(feed.headers()["referrer-policy"], "no-referrer");
    let text = feed.text().await.unwrap();
    assert!(text.contains("<rss"));
    assert!(!text.contains("resource_token"));
    assert_eq!(client.head(&feed_url).send().await.unwrap().status(), 200);
    assert_eq!(client.get(&feed_url).bearer_auth(access).send().await.unwrap().status(), 401);
    assert_eq!(
      client.get(format!("{feed_url}&resource_token={feed_token}")).send().await.unwrap().status(),
      401
    );
    assert_eq!(
      client
        .get(format!("{base}/api/feeds/rss/native-feed-B?resource_token={feed_token}"))
        .send()
        .await
        .unwrap()
        .status(),
      401
    );
    let foreign = derive("feeds/rss/native-feed-B", issued, issued + 300);
    assert_eq!(
      client
        .get(format!("{base}/api/feeds/rss/native-feed-B?resource_token={foreign}"))
        .send()
        .await
        .unwrap()
        .status(),
      403
    );
    assert_eq!(
      client
        .get(format!("{base}/api/feeds/rss/native-feed-A"))
        .bearer_auth(access)
        .send()
        .await
        .unwrap()
        .status(),
      200
    );
    let path = "image/posters/cover.png";
    let source = ctx.storage().build_subscriber_path(owner.try_into().unwrap(), path);
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(16, 16, image::Rgb([12, 34, 56])));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    ctx.storage().write(source, bytes::Bytes::from(out.into_inner())).await.unwrap();
    let route = format!("{base}/api/static/subscribers/{owner}/{path}");
    let token = derive(path, issued, issued + 300);
    let native = format!("{route}?resource_token={token}");
    assert_eq!(client.get(&route).send().await.unwrap().status(), 401);
    assert_eq!(client.get(&route).bearer_auth(access).send().await.unwrap().status(), 200);
    let response = client.get(&native).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-cache, no-transform");
    assert_eq!(response.headers()["cross-origin-resource-policy"], "cross-origin");
    let etag = response.headers()["etag"].clone();
    assert_eq!(client.get(&native).header("If-None-Match", etag.clone()).send().await.unwrap().status(), 304);
    let head = client.head(&native).send().await.unwrap();
    assert_eq!(head.status(), 200);
    assert!(head.bytes().await.unwrap().is_empty());
    assert_eq!(client.get(&native).header("Range", "bytes=0-3").send().await.unwrap().status(), 206);
    assert_eq!(client.get(&native).bearer_auth(access).send().await.unwrap().status(), 401);
    assert_eq!(client.get(format!("{native}&resource_token={token}")).send().await.unwrap().status(), 401);
    for bad in [
      URL_SAFE_NO_PAD.encode(grant["grant"].as_str().unwrap()),
      derive(path, issued, issued + 301),
      derive(path, issued, issued),
    ] {
      assert_eq!(
        client
          .get(format!("{route}?resource_token={bad}"))
          .header("If-None-Match", etag.clone())
          .send()
          .await
          .unwrap()
          .status(),
        401
      );
    }
    let clock = u64::try_from(chrono::Utc::now().timestamp()).unwrap();
    let expiring = derive(path, clock, clock + 1);
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert_eq!(
      client
        .get(format!("{route}?resource_token={expiring}"))
        .header("If-None-Match", etag.clone())
        .send()
        .await
        .unwrap()
        .status(),
      401
    );
    for other in [
      format!("{base}/api/static/subscribers/{}/{path}", owner + 1),
      format!("{base}/api/static/subscribers/{owner}/image/posters/missing.png"),
      format!("{base}/api/static/subscribers/{owner}/image/posters/%252e%252e"),
      format!("{base}/api/auth/user-info"),
    ] {
      assert_eq!(client.get(format!("{other}?resource_token={token}")).send().await.unwrap().status(), 401);
    }
    assert_eq!(
      client
        .post(format!("{issue}?resource_token={token}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap()
        .status(),
      401
    );
  })
  .catch_unwind()
  .await;
  shutdown.send(()).unwrap();
  served.await.unwrap().unwrap();
  ctx.auth().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = outcome {
    std::panic::resume_unwind(panic);
  }
}
