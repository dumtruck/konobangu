//! The embedded issuer and recorder's sole resource-server model share one
//! identity contract.
use std::sync::Arc;

use axum::{Router, body::Body};
use futures::FutureExt;
use recorder::{
  app::AppContextTrait,
  auth::AuthService,
  database::DatabaseService,
  test_utils::app::TestingAppContext,
  web::controller::{ControllerTrait, auth},
};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use tower::ServiceExt;

use super::support::{RlsFixture, bundled_tokens};

#[tokio::test]
async fn bundled_oidc_maps_the_trusted_local_subject_without_accepting_basic_on_the_api() {
  let fixture = RlsFixture::new().await;
  let mut config = fixture.bundled_auth("http://127.0.0.1:5001");
  config.webui.allowed_origins.push(securitydept_utils::redirect::RedirectTargetRule::Strict {
    value: "http://127.0.0.1:5002".into(),
  });
  let mut database = fixture.bootstrap.config.clone();
  database.uri = fixture.app_uri.clone();
  database.auto_migrate = false;
  let ctx: Arc<dyn AppContextTrait> = Arc::new(
    TestingAppContext::builder()
      .db(DatabaseService::from_config(database).await.unwrap())
      .auth(AuthService::from_conf(config.clone()).await.unwrap())
      .build(),
  );
  let router = auth::create(ctx.clone()).await.unwrap().apply_to(Router::new()).with_state(ctx.clone());
  let result = std::panic::AssertUnwindSafe(async {
    let response = router
      .clone()
      .oneshot(
        http::Request::builder()
          .uri("/api/auth/config")
          .header("origin", "http://127.0.0.1:5001")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), 200);
    let public: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
    assert_eq!(public["provider"], "oidc");
    assert_eq!(public["issuerUrl"], "http://127.0.0.1:5001/auth-provider");
    assert!(public["pkceEnabled"].as_bool().unwrap());
    // An application origin rule does not register an OAuth callback.
    let unregistered = router
      .clone()
      .oneshot(
        http::Request::builder()
          .uri("/api/auth/config")
          .header("origin", "http://127.0.0.1:5002")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(unregistered.status(), 403);
    let tokens = bundled_tokens(ctx.auth()).await;
    let access = tokens["access_token"].as_str().unwrap();
    let mut parts = http::Request::builder()
      .header("authorization", format!("Bearer {access}"))
      .body(())
      .unwrap()
      .into_parts()
      .0;
    let principal = ctx.auth().extract_user_info(&mut parts).await.unwrap();
    assert!(principal.is_admin);
    assert_eq!(principal.subscriber_auth.subscriber_id, 1);
    assert_eq!(principal.subscriber_auth.pid, "local-admin");
    assert_eq!(principal.subscriber_auth.auth_type, recorder::models::auth::AuthType::Oidc);
    let identity = router
      .clone()
      .oneshot(
        http::Request::builder()
          .uri("/api/auth/user-info")
          .header("authorization", format!("Bearer {access}"))
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(identity.status(), 200);
    let identity: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(identity.into_body(), 100_000).await.unwrap()).unwrap();
    assert_eq!(identity["authType"], "oidc");
    assert_eq!(identity["subject"], "local-admin");
    for credential in [
      "Basic Zml4dHVyZS1hZG1pbjpmaXh0dXJlLXBhc3N3b3Jk".to_owned(),
      format!("Bearer {}", tokens["id_token"].as_str().unwrap()),
      format!("Bearer {}", tokens["refresh_token"].as_str().unwrap()),
    ] {
      let denied = router
        .clone()
        .oneshot(
          http::Request::builder()
            .uri("/api/auth/user-info")
            .header("authorization", credential)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(denied.status(), 401);
      assert_eq!(denied.headers()["www-authenticate"], "Bearer");
      assert!(
        !denied
          .headers()
          .get_all("www-authenticate")
          .iter()
          .any(|value| value.to_str().unwrap().starts_with("Basic"))
      );
    }
    let forbidden_source = router
      .clone()
      .oneshot(
        http::Request::builder()
          .uri("/api/auth/config")
          .header("origin", "https://not-registered.example")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(forbidden_source.status(), 403);
    // Reopening the Provider does not create another application owner or lose
    // its key.
    let previous_jwks = ctx.auth().bundled_provider().unwrap().jwks();
    let restarted = AuthService::from_conf(config).await.unwrap();
    assert_eq!(
      serde_json::to_value(previous_jwks).unwrap(),
      serde_json::to_value(restarted.bundled_provider().unwrap().jwks()).unwrap()
    );
    let verified = restarted.oidc.verifier.authenticate(access).await.unwrap();
    assert_eq!(verified.subject, "local-admin");
    restarted.close().await.unwrap();
    let count: i64 = fixture
      .bootstrap
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT count(*) FROM auth_identity.auth WHERE issuer='http://127.0.0.1:5001/auth-provider' AND pid='local-admin' AND subscriber_id=1",
      ))
      .await
      .unwrap()
      .unwrap()
      .try_get_by_index(0)
      .unwrap();
    assert_eq!(count, 1);
  })
  .catch_unwind()
  .await;
  ctx.auth().close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
  if let Err(panic) = result {
    std::panic::resume_unwind(panic);
  }
}
