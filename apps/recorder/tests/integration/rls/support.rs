use recorder::{
  database::{DatabaseConfig, DatabaseService},
  migrations::access::{RuntimeLogins, grant_runtime_memberships},
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

#[allow(
  dead_code,
  reason = "Shared RLS tests and native smoke receipts use this fixture metadata; the HTTP support server does not"
)]
pub const SUBSCRIBER_A: i32 = 1001;
#[allow(
  dead_code,
  reason = "Shared RLS tests and native smoke receipts use this fixture metadata; the HTTP support server does not"
)]
pub const SUBSCRIBER_B: i32 = 1002;

// Each fixture owns its container. No development URL or container pruning is
// used.
pub struct RlsFixture {
  pub app: DatabaseConnection,
  pub bootstrap: DatabaseService,
  pub app_uri: String,
  pub identity_uri: String,
  pub queue_uri: String,
  pub provider_uri: String,
  _container: ContainerAsync<Postgres>,
}

impl RlsFixture {
  #[allow(
    dead_code,
    reason = "Shared RLS tests and native smoke receipts use this fixture metadata; the HTTP support server does not"
  )]
  pub fn container_id(&self) -> &str {
    self._container.id()
  }
  #[allow(
    dead_code,
    reason = "Native resource RLS suites configure a runtime pool; the shared HTTP support binary does not"
  )]
  pub fn runtime_config(&self, auth: recorder::auth::AuthConfig) -> recorder::app::AppConfig {
    let mut database = self.bootstrap.config.clone();
    database.uri = self.app_uri.clone();
    database.auto_migrate = false;
    figment::Figment::new()
      .merge(recorder::app::AppConfig::default_provider())
      .merge(figment::providers::Serialized::defaults(serde_json::json!({"database":database,"auth":auth})))
      .extract()
      .unwrap()
  }

  pub async fn insert_rss_feed(&self, owner: i32, subscription: i32, token: &str) -> recorder::models::feeds::Model {
    use recorder::models::feeds;
    use sea_orm::{ActiveModelTrait, Set};
    feeds::ActiveModel {
      subscriber_id: Set(Some(owner)),
      subscription_id: Set(Some(subscription)),
      token: Set(token.into()),
      feed_type: Set(feeds::FeedType::Rss),
      feed_source: Set(feeds::FeedSource::SubscriptionEpisode),
      ..Default::default()
    }
    .insert(&self.bootstrap)
    .await
    .unwrap()
  }

  #[allow(
    dead_code,
    reason = "Native resource RLS suites create subscriptions; the shared HTTP support binary supplies existing IDs"
  )]
  pub async fn seed_rss_feed(&self, owner: i32, token: &str) -> recorder::models::feeds::Model {
    use recorder::models::subscriptions;
    use sea_orm::{ActiveModelTrait, Set};
    let subscription = subscriptions::ActiveModel {
      display_name: Set("Native RSS fixture".into()),
      subscriber_id: Set(owner),
      category: Set(subscriptions::SubscriptionCategory::MikanSubscriber),
      source_url: Set("https://example.test/feed".into()),
      enabled: Set(false),
      ..Default::default()
    }
    .insert(&self.bootstrap)
    .await
    .unwrap();
    self.insert_rss_feed(owner, subscription.id, token).await
  }

  pub async fn new() -> Self {
    let container = Postgres::default()
      .with_db_name("rls_test")
      .with_user("migration_owner")
      .with_password("migration_test_password")
      .with_tag("18-alpine")
      .start()
      .await
      .expect("PostgreSQL testcontainer must start");
    let host = container.get_host().await.expect("container host");
    let port = container.get_host_port_ipv4(5432).await.expect("PostgreSQL port");
    let bootstrap = DatabaseService::from_migration_config(DatabaseConfig {
      uri: format!("postgres://migration_owner:migration_test_password@{host}:{port}/rls_test"),
      enable_logging: false,
      min_connections: 1,
      max_connections: 1,
      connect_timeout: 10000,
      idle_timeout: 60000,
      acquire_timeout: Some(10000),
      auto_migrate: true,
      legacy_oidc_issuer: None,
      migration_database_uri: None,
    })
    .await
    .expect("existing Apalis and application migrations must apply");
    bootstrap
      .execute_unprepared(
        "CREATE ROLE rls_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'rls_test_password';
       CREATE ROLE auth_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'identity_test_password';
       CREATE ROLE task_control_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'queue_test_password';
       CREATE ROLE auth_provider_app LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD 'provider_test_password'",
      )
      .await
      .expect("fixture-owned login credentials");
    grant_runtime_memberships(
      bootstrap.as_ref(),
      &RuntimeLogins {
        app_scoped: "rls_app".into(),
        auth: "auth_app".into(),
        task_control: Some("task_control_app".into()),
        auth_provider: Some("auth_provider_app".into()),
      },
    )
    .await
    .expect("provision production access contracts for fixture logins");
    bootstrap
      .execute_unprepared(
        "INSERT INTO subscribers (id, display_name) VALUES (1001, 'Subscriber A'), (1002, 'Subscriber B');
       INSERT INTO subscriptions (display_name, subscriber_id, \
         source_url, enabled, category) VALUES
         ('A seed', 1001, 'https://example.test/a', true, 'mikan_subscriber'),
         ('B seed', 1002, 'https://example.test/b', \
         true, 'mikan_subscriber');",
      )
      .await
      .expect("fixture passwords and two subscribers' resources");
    let mut options = ConnectOptions::new(format!("postgres://rls_app:rls_test_password@{host}:{port}/rls_test"));
    // A single physical connection makes identity reuse deterministic.
    options.min_connections(1).max_connections(1).sqlx_logging(false);
    recorder::database::roles::restrict_pool(&mut options, recorder::database::roles::APP_SCOPED_ACCESS_ROLE);
    let app = Database::connect(options).await.expect("ordinary application role connects");
    Self {
      app,
      bootstrap,
      app_uri: format!("postgres://rls_app:rls_test_password@{host}:{port}/rls_test"),
      provider_uri: format!("postgres://auth_provider_app:provider_test_password@{host}:{port}/rls_test"),
      queue_uri: format!("postgres://task_control_app:queue_test_password@{host}:{port}/rls_test"),
      identity_uri: format!("postgres://auth_app:identity_test_password@{host}:{port}/rls_test"),
      _container: container,
    }
  }

  #[allow(dead_code, reason = "Shared fixture is also included by browser support and SQL-only suites")]
  pub fn bundled_auth(&self, base: &str) -> recorder::auth::AuthConfig {
    serde_json::from_value(serde_json::json!({
      "provider": { "type": "bundled", "issuer": format!("{base}/auth-provider"),
        "client": { "client_id": "konobangu-test", "redirect_uris": [format!("{base}/auth/callback")] },
        "credentials": { "username": "fixture-admin", "password": "fixture-password" },
        "signing": { "private_key_pem": fixture_private_key() }, "database": { "url": self.provider_uri },
      },
      "webui": { "allowed_origins": [{"type":"strict", "value":base}] },
      "identity": { "database_url": self.identity_uri },
    }))
    .unwrap()
  }

  pub async fn close(self) {
    self.app.close().await.expect("close application pool");
    self.bootstrap.as_ref().clone().close().await.expect("close bootstrap pool");
    self._container.rm().await.expect("remove owned PostgreSQL fixture");
  }
}

#[allow(dead_code, reason = "Shared fixture is also included by browser support and SQL-only suites")]
pub fn fixture_private_key() -> &'static str {
  static KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
  KEY.get_or_init(|| {
    let output = std::process::Command::new("node")
      .args([
        "--input-type=module",
        "-e",
        "import {generateKeyPairSync} from 'node:crypto'; \
         process.stdout.write(generateKeyPairSync('rsa',{modulusLength:2048,privateKeyEncoding:{type:'pkcs8',format:'pem'},publicKeyEncoding:{type:'spki',\
         format:'pem'}}).privateKey)",
      ])
      .output()
      .expect("mise Node is required for the test signing key");
    assert!(output.status.success(), "fixture key generation must succeed");
    String::from_utf8(output.stdout).unwrap()
  })
}

/// Exercises the Provider's production router; business tests never fabricate a
/// JWT.
#[allow(dead_code, reason = "Shared fixture is also included by browser support and SQL-only suites")]
pub async fn bundled_tokens(auth: &recorder::auth::AuthService) -> serde_json::Value {
  use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
  };
  use sha2::{Digest, Sha256};
  use tower::ServiceExt;
  let provider = auth.bundled_provider().expect("bundled fixture");
  let config = provider.config();
  let verifier = "fixture-proof-key-verifier-with-at-least-forty-three-characters";
  let mut authorization = url::Url::parse(&config.endpoint("authorize")).unwrap();
  authorization.query_pairs_mut().extend_pairs([
    ("client_id", config.client.client_id.as_str()),
    ("redirect_uri", config.client.redirect_uris[0].as_str()),
    ("response_type", "code"),
    ("scope", "openid profile offline_access konobangu:api"),
    ("code_challenge_method", "S256"),
    ("code_challenge", URL_SAFE_NO_PAD.encode(Sha256::digest(verifier)).as_str()),
    ("state", "fixture-state"),
    ("nonce", "fixture-nonce"),
  ]);
  let router = provider.clone().router();
  let request = |uri: &str| {
    http::Request::builder()
      .uri(uri)
      .extension(axum::extract::ConnectInfo("127.0.0.1:12345".parse::<std::net::SocketAddr>().unwrap()))
  };
  let path = |url: &url::Url| {
    format!(
      "{}{}",
      url.path().strip_prefix("/auth-provider").unwrap(),
      url.query().map(|q| format!("?{q}")).unwrap_or_default()
    )
  };
  let initial = router
    .clone()
    .oneshot(request(&path(&authorization)).body(axum::body::Body::empty()).unwrap())
    .await
    .unwrap();
  assert!(initial.status().is_redirection(), "valid authorization must reach login");
  let login = url::Url::parse(initial.headers()["location"].to_str().unwrap())
    .or_else(|_| url::Url::parse(&config.issuer).unwrap().join(initial.headers()["location"].to_str().unwrap()))
    .unwrap();
  let authenticated = router
    .clone()
    .oneshot(
      request(&path(&login))
        .header(
          "authorization",
          format!(
            "Basic {}",
            STANDARD.encode(format!("{}:{}", config.credentials.username, config.credentials.password))
          ),
        )
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert!(authenticated.status().is_redirection(), "valid Provider login must complete authorization");
  let callback = url::Url::parse(authenticated.headers()["location"].to_str().unwrap()).unwrap();
  let code = callback
    .query_pairs()
    .find(|(key, _)| key == "code")
    .expect("authorization code")
    .1
    .into_owned();
  let body = url::form_urlencoded::Serializer::new(String::new())
    .extend_pairs([
      ("grant_type", "authorization_code"),
      ("code", code.as_str()),
      ("client_id", config.client.client_id.as_str()),
      ("redirect_uri", config.client.redirect_uris[0].as_str()),
      ("code_verifier", verifier),
    ])
    .finish();
  let response = router
    .oneshot(
      request("/token")
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(axum::body::Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), http::StatusCode::OK, "valid code and PKCE must exchange");
  let token: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 1_000_000).await.unwrap()).unwrap();
  token
}

#[allow(dead_code, reason = "Shared fixture is also included by browser support and SQL-only suites")]
pub async fn bundled_access_token(auth: &recorder::auth::AuthService) -> String {
  bundled_tokens(auth).await["access_token"].as_str().unwrap().into()
}
