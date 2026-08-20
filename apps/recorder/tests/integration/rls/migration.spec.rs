use recorder::{
  database::DatabaseService,
  migrations::{
    Migrator, MigratorTrait,
    access::{RuntimeLogins, grant_runtime_memberships},
  },
};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

use super::support::RlsFixture;

#[tokio::test]
async fn configured_login_capabilities_enforce_current_access_boundaries() {
  use std::sync::Arc;

  use recorder::{
    app::AppContextTrait,
    auth::connect_identity_db,
    database::rls::bind_subscriber_to_transaction,
    task::{TaskConfig, TaskService},
    test_utils::app::TestingAppContext,
  };

  let fixture = RlsFixture::new().await;
  let roles = RuntimeLogins {
    app_scoped: "app scoped \"测试".into(),
    auth: "app scoped \"测试".into(),
    task_control: Some("app scoped \"测试".into()),
    auth_provider: None,
  };
  fixture
    .bootstrap
    .execute_unprepared("CREATE ROLE \"app scoped \"\"测试\" LOGIN NOSUPERUSER NOBYPASSRLS")
    .await
    .unwrap();
  let uri = |role: &str| {
    let mut url = url::Url::parse(&fixture.bootstrap.config.uri).unwrap();
    url.set_username(role).unwrap();
    url.set_password(Some("role-test-password")).unwrap();
    url.to_string()
  };
  let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures")
    .join(format!("role-cli-{}", uuid::Uuid::new_v4()));
  std::fs::create_dir_all(&directory).unwrap();
  let config_path = directory.join("config.json");
  let dotenv_path = directory.join("fixture.env");
  std::fs::write(&dotenv_path, "").unwrap();
  std::fs::write(&config_path, serde_json::to_vec(&serde_json::json!({
    "database": {"url": uri(&roles.app_scoped), "migration": {"url": fixture.bootstrap.config.uri, "auto_run": false}},
    "auth": {"provider": {"type":"bundled", "issuer":"https://issuer.test/auth-provider", "client":{"client_id":"fixture","redirect_uris":["http://127.0.0.1:5001/auth/callback"]}, "credentials":{"username":"fixture","password":"fixture"}, "signing":{"private_key_pem":"fixture-key"}}, "webui": {"allowed_origins": [{"type":"strict", "value": "http://127.0.0.1:5001"}]}, "identity": {"database_url": uri(&roles.auth)}},
    "scheduler": {"database": {"url": uri(roles.task_control.as_deref().unwrap())}},
  })).unwrap()).unwrap();
  let cli = std::process::Command::new(env!("CARGO_BIN_EXE_migrate-up"))
    .env_clear()
    .args(["--environment", "testing", "--config-file"])
    .arg(&config_path)
    .arg("--dotenv-file")
    .arg(&dotenv_path)
    .output()
    .unwrap();
  std::fs::remove_dir_all(&directory).unwrap();
  assert!(cli.status.success(), "role provisioning CLI: {}", String::from_utf8_lossy(&cli.stderr));
  // Reentry reapplies grants without resetting independently provisioned
  // passwords.
  for role in [&roles.app_scoped] {
    let password_sql: String = fixture
      .bootstrap
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT format('ALTER ROLE %I PASSWORD %L', $1::text, $2::text) AS sql",
        [role.clone().into(), "role-test-password".into()],
      ))
      .await
      .unwrap()
      .unwrap()
      .try_get("", "sql")
      .unwrap();
    fixture.bootstrap.execute_unprepared(&password_sql).await.unwrap();
  }
  grant_runtime_memberships(fixture.bootstrap.as_ref(), &roles).await.unwrap();
  let mut scoped = fixture.bootstrap.config.clone();
  scoped.auto_migrate = true;
  scoped.migration_database_uri = Some(fixture.bootstrap.config.uri.clone());
  scoped.uri = uri(&roles.app_scoped);
  let ctx = Arc::new(TestingAppContext::builder().db(DatabaseService::from_config(scoped).await.unwrap()).build());
  assert!(
    ctx
      .db()
      .query_all_raw(Statement::from_string(DbBackend::Postgres, "SELECT id FROM subscriptions"))
      .await
      .unwrap()
      .is_empty()
  );
  let txn = ctx.db().begin().await.unwrap();
  bind_subscriber_to_transaction(&txn, super::support::SUBSCRIBER_A).await.unwrap();
  assert_eq!(
    txn
      .query_all_raw(Statement::from_string(DbBackend::Postgres, "SELECT id FROM subscriptions"))
      .await
      .unwrap()
      .len(),
    1
  );
  txn.rollback().await.unwrap();
  let auth_db = connect_identity_db(&uri(&roles.auth)).await.unwrap();
  assert!(auth_db.execute_unprepared("SELECT id FROM task_runs").await.is_err());
  let config = TaskConfig {
    queue_database_uri: Some(uri(roles.task_control.as_deref().unwrap())),
    ..Default::default()
  };
  let task = TaskService::from_config_and_ctx(config, ctx.clone()).await.unwrap();
  task
    .queue_database()
    .unwrap()
    .execute_unprepared("SELECT id FROM task_runs; SELECT id FROM cron; SELECT id FROM apalis.jobs")
    .await
    .unwrap();
  assert!(task.queue_database().unwrap().execute_unprepared("SELECT id FROM subscriptions").await.is_err());
  assert!(
    task
      .queue_database()
      .unwrap()
      .execute_unprepared("SELECT id FROM auth_identity.auth")
      .await
      .is_err()
  );
  task.close().await.unwrap();
  auth_db.close().await.unwrap();
  ctx.db().as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn legacy_identity_migration_requires_attribution_and_preserves_shared_owners() {
  let fixture = RlsFixture::new().await;
  fixture.bootstrap.execute_unprepared("CREATE DATABASE legacy_test").await.unwrap();
  let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../temp/verification/fixtures")
    .join(format!("migration-cli-{}", uuid::Uuid::new_v4()));
  std::fs::create_dir_all(&directory).unwrap();
  let config_path = directory.join("config.json");
  let dotenv_path = directory.join("fixture.env");
  std::fs::write(&dotenv_path, "").unwrap();
  std::fs::write(&config_path, serde_json::to_vec(&serde_json::json!({
    "database": fixture.bootstrap.config,
    "auth": { "provider": {"type":"bundled", "issuer":"https://issuer.test/auth-provider", "client":{"client_id":"fixture","redirect_uris":["http://127.0.0.1:5001/auth/callback"]}, "credentials":{"username":"fixture","password":"fixture"}, "signing":{"private_key_pem":"fixture-key"}}, "webui": {"allowed_origins": [{"type":"strict", "value": "http://127.0.0.1:5001"}]}, "identity": {"database_url": fixture.identity_uri} },
    "server": { "host": "127.0.0.1" }, "logger": { "enable": false, "level": "off", "format": "compact" }
  })).unwrap()).unwrap();
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600)).unwrap();
  }
  let cli = std::process::Command::new(env!("CARGO_BIN_EXE_migrate-up"))
    .env_clear()
    .args(["--environment", "production", "--config-file"])
    .arg(&config_path)
    .arg("--dotenv-file")
    .arg(&dotenv_path)
    .output()
    .unwrap();
  std::fs::remove_dir_all(&directory).unwrap();
  assert!(cli.status.success(), "documented owner migration CLI must succeed: {:?}", cli.status);
  let mut config = fixture.bootstrap.config.clone();
  config.uri = config.uri.replace("/rls_test", "/legacy_test");
  config.auto_migrate = false;
  let legacy = DatabaseService::from_migration_config(config.clone()).await.unwrap();
  Migrator::up(
    legacy.as_ref(),
    Some(
      Migrator::migrations()
        .iter()
        .position(|migration| migration.name() == "m20261003_000001_identity_session")
        .unwrap() as u32,
    ),
  )
  .await
  .unwrap();
  legacy.execute_unprepared("INSERT INTO auth(pid, auth_type, subscriber_id) VALUES ('existing-subject', 'oidc', 1), ('shared-subject', 'oidc', 1); INSERT INTO subscriptions(display_name, subscriber_id, source_url, enabled, category) VALUES ('legacy subscription', 1, 'https://example.test/legacy', true, 'mikan_subscriber')").await.unwrap();
  assert!(
    legacy.migrate_up().await.is_err(),
    "unknown issuer attribution must stop before changing identity data"
  );
  let untouched = legacy
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT count(*) FROM public.auth WHERE auth_type = 'oidc'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(untouched.try_get_by_index::<i64>(0).unwrap(), 2);
  let no_schema = legacy
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname = 'auth_identity')",
    ))
    .await
    .unwrap()
    .unwrap();
  assert!(!no_schema.try_get_by_index::<bool>(0).unwrap());
  config.legacy_oidc_issuer = Some("https://legacy-issuer.example".into());
  let attributed = DatabaseService::from_migration_config(config).await.unwrap();
  attributed.migrate_up().await.unwrap();
  attributed.migrate_up().await.unwrap();
  // Fail a later queue initialization stage, retain the committed application
  // migrations, then retry from that stopped state.
  attributed
    .execute_unprepared("DROP SCHEMA apalis CASCADE; CREATE SCHEMA apalis; CREATE TABLE apalis.jobs (bad integer)")
    .await
    .unwrap();
  assert!(attributed.migrate_up().await.is_err());
  attributed.execute_unprepared("DROP SCHEMA apalis CASCADE").await.unwrap();
  attributed.migrate_up().await.unwrap();
  let rows = attributed
    .query_all_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT issuer, subscriber_id FROM auth_identity.auth WHERE auth_type = 'oidc' ORDER BY pid",
    ))
    .await
    .unwrap();
  assert_eq!(rows.len(), 2);
  for row in rows {
    assert_eq!(row.try_get::<String>("", "issuer").unwrap(), "https://legacy-issuer.example");
    assert_eq!(row.try_get::<i32>("", "subscriber_id").unwrap(), 1);
  }
  let basic = attributed
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT issuer IS NULL FROM auth_identity.auth WHERE auth_type = 'basic'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert!(basic.try_get_by_index::<bool>(0).unwrap());
  let subscription = attributed
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT subscriber_id FROM subscriptions WHERE display_name = 'legacy subscription'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert_eq!(subscription.try_get_by_index::<i32>(0).unwrap(), 1);
  attributed
    .execute_unprepared(
      r#"INSERT INTO task_runs(id,kind,subscriber_id,subscription_id,payload,task_type,status) VALUES
         ('finished-run','subscriber_task',1,1,'{"task_type":"sync_one_subscription_sources","subscriber_id":1,"subscription_id":1}','sync_one_subscription_sources','Done')"#,
    )
    .await
    .unwrap();
  let transaction = attributed.begin().await.unwrap();
  recorder::database::rls::bind_subscriber_to_transaction(&transaction, 1).await.unwrap();
  let filter = sea_orm::Condition::all().add(sea_orm::ColumnTrait::eq(&recorder::models::subscriptions::Column::Id, 1));
  assert_eq!(recorder::models::subscriptions::operation::delete_owned(&transaction, filter).await.unwrap(), 1);
  transaction.commit().await.unwrap();
  let history = attributed
    .query_one_raw(Statement::from_string(
      DbBackend::Postgres,
      "SELECT subscription_id IS NULL AS detached,payload->>'subscription_id' AS original FROM task_runs WHERE id='finished-run'",
    ))
    .await
    .unwrap()
    .unwrap();
  assert!(history.try_get::<bool>("", "detached").unwrap());
  assert_eq!(history.try_get::<String>("", "original").unwrap(), "1");
  attributed.migrate_up().await.unwrap();
  assert!(
    Migrator::down(attributed.as_ref(), Some(1)).await.is_err(),
    "unsafe down must require backup restoration"
  );
  legacy.as_ref().clone().close().await.unwrap();
  attributed.as_ref().clone().close().await.unwrap();
  fixture.close().await;
}

#[tokio::test]
async fn single_createrole_owner_bootstraps_restricted_pools_without_manual_provisioning() {
  use std::sync::Arc;

  use figment::{Figment, providers::Serialized};
  use recorder::{
    app::AppConfig,
    auth::connect_identity_db,
    database::roles::{APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, CAPABILITY_ROLES, TASK_CONTROL_ACCESS_ROLE, set_local_role},
    task::TaskService,
    test_utils::app::TestingAppContext,
  };
  use testcontainers::{ImageExt, runners::AsyncRunner};
  use testcontainers_modules::postgres::Postgres;

  let container = Postgres::default().with_tag("18-alpine").start().await.unwrap();
  let host = container.get_host().await.unwrap();
  let port = container.get_host_port_ipv4(5432).await.unwrap();
  let bootstrap = sea_orm::Database::connect(format!("postgres://postgres:postgres@{host}:{port}/postgres"))
    .await
    .unwrap();
  bootstrap
    .execute_unprepared("CREATE ROLE deployment_owner LOGIN CREATEROLE NOSUPERUSER NOBYPASSRLS PASSWORD 'deployment-password'")
    .await
    .unwrap();
  bootstrap
    .execute_unprepared("CREATE DATABASE deployment_test OWNER deployment_owner")
    .await
    .unwrap();
  let uri = format!("postgres://deployment_owner:deployment-password@{host}:{port}/deployment_test");
  let mut config: AppConfig = Figment::new()
    .merge(AppConfig::default_provider())
    .merge(Serialized::defaults(serde_json::json!({
      "database": {"url": uri, "pool": {"min_connections": 1, "max_connections": 1}},
      "auth": {"webui": {"allowed_origins": [{"type":"strict", "value": "http://127.0.0.1:5001"}]},
               "provider": {"type":"bundled", "issuer":"https://issuer.test/auth-provider", "client":{"client_id":"fixture","redirect_uris":["http://127.0.0.1:5001/auth/callback"]}, "credentials":{"username":"fixture","password":"fixture"}, "signing":{"private_key_pem":"fixture-key"}}},
    })))
    .extract()
    .unwrap();
  config.resolve_database_urls();
  assert!(config.database.auto_migrate);
  assert_eq!(config.auth.identity.database_url, config.database.uri);
  assert_eq!(config.task.queue_database_uri.as_deref(), Some(config.database.uri.as_str()));
  let db = DatabaseService::from_app_config(&config).await.unwrap();
  for capability in CAPABILITY_ROLES {
    let row = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT rolcanlogin, rolsuper, rolbypassrls, pg_has_role(session_user, oid, 'USAGE') AS inherited, pg_has_role(session_user, oid, 'SET') AS \
         switchable FROM pg_roles WHERE rolname=$1",
        [capability.into()],
      ))
      .await
      .unwrap()
      .unwrap();
    assert!(!row.try_get::<bool>("", "rolcanlogin").unwrap());
    assert!(!row.try_get::<bool>("", "rolsuper").unwrap());
    assert!(!row.try_get::<bool>("", "rolbypassrls").unwrap());
    assert!(!row.try_get::<bool>("", "inherited").unwrap());
    assert!(row.try_get::<bool>("", "switchable").unwrap());
  }
  async fn role(db: &impl ConnectionTrait) -> (String, i32) {
    let row = db
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT current_user::text AS role, pg_backend_pid() AS pid",
      ))
      .await
      .unwrap()
      .unwrap();
    (row.try_get("", "role").unwrap(), row.try_get("", "pid").unwrap())
  }
  let baseline = role(&db).await;
  assert_eq!(baseline.0, APP_SCOPED_ACCESS_ROLE);
  for commit in [true, false] {
    let txn = db.begin().await.unwrap();
    set_local_role(&txn, AUTH_ACCESS_ROLE).await.unwrap();
    assert_eq!(role(&txn).await, (AUTH_ACCESS_ROLE.to_string(), baseline.1));
    if commit {
      txn.commit().await.unwrap();
    } else {
      txn.rollback().await.unwrap();
    }
    assert_eq!(role(&db).await, baseline);
  }
  // A new app startup reruns idempotent migrations and grants using the same
  // non-superuser owner credential.
  let restarted = DatabaseService::from_app_config(&config).await.unwrap();
  assert_eq!(role(&restarted).await.0, APP_SCOPED_ACCESS_ROLE);
  restarted.as_ref().clone().close().await.unwrap();
  let auth_db = connect_identity_db(&config.auth.identity.database_url).await.unwrap();
  assert_eq!(role(&auth_db).await.0, AUTH_ACCESS_ROLE);
  assert!(auth_db.execute_unprepared("SELECT id FROM subscriptions").await.is_err());
  let ctx = Arc::new(TestingAppContext::builder().db(db).build());
  let task = TaskService::from_config_and_ctx(config.task, ctx.clone()).await.unwrap();
  assert_eq!(role(task.queue_database().unwrap()).await.0, TASK_CONTROL_ACCESS_ROLE);
  assert!(
    task
      .queue_database()
      .unwrap()
      .execute_unprepared("SELECT id FROM auth_identity.auth")
      .await
      .is_err()
  );
  task.close().await.unwrap();
  auth_db.close().await.unwrap();
  use recorder::app::AppContextTrait;
  ctx.db().as_ref().clone().close().await.unwrap();
  bootstrap.close().await.unwrap();
  container.rm().await.unwrap();
}

#[tokio::test]
async fn provider_protocol_storage_has_an_independent_capability_boundary() {
  use konobangu_database::{
    PoolConfig, connect,
    roles::{APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, AUTH_PROVIDER_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE},
  };
  let fixture = RlsFixture::new().await;
  for (uri, capability) in [
    (fixture.app_uri.as_str(), APP_SCOPED_ACCESS_ROLE),
    (fixture.identity_uri.as_str(), AUTH_ACCESS_ROLE),
    (fixture.queue_uri.as_str(), TASK_CONTROL_ACCESS_ROLE),
    (fixture.provider_uri.as_str(), AUTH_PROVIDER_ACCESS_ROLE),
  ] {
    let db = connect(
      uri,
      &PoolConfig {
        min_connections: 1,
        max_connections: 1,
        ..Default::default()
      },
      Some(capability),
    )
    .await
    .unwrap();
    let boundary = db
      .query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        "SELECT current_user::text AS role, has_schema_privilege(current_user,'auth_provider','USAGE') AS provider_schema, EXISTS(SELECT 1 FROM pg_class c \
         JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='auth_provider' AND c.relkind='r') AS provider_tables, EXISTS(SELECT 1 FROM pg_class c \
         JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='auth_provider' AND c.relkind='r' AND \
         has_table_privilege(current_user,c.oid,'SELECT,INSERT,UPDATE,DELETE')) AS provider_access, \
         has_schema_privilege(current_user,'auth_identity','USAGE') AS identity_schema, has_schema_privilege(current_user,'apalis','USAGE') AS task_schema",
      ))
      .await
      .unwrap()
      .unwrap();
    assert_eq!(boundary.try_get::<String>("", "role").unwrap(), capability);
    assert!(boundary.try_get::<bool>("", "provider_tables").unwrap());
    assert_eq!(
      boundary.try_get::<bool>("", "provider_schema").unwrap(),
      capability == AUTH_PROVIDER_ACCESS_ROLE
    );
    assert_eq!(
      boundary.try_get::<bool>("", "provider_access").unwrap(),
      capability == AUTH_PROVIDER_ACCESS_ROLE
    );
    if capability == AUTH_PROVIDER_ACCESS_ROLE {
      assert!(!boundary.try_get::<bool>("", "identity_schema").unwrap());
      assert!(!boundary.try_get::<bool>("", "task_schema").unwrap());
      for forbidden in ["SELECT id FROM public.subscriptions", "SELECT id FROM auth_identity.auth"] {
        assert!(db.execute_unprepared(forbidden).await.is_err(), "Provider cannot acquire another capability");
      }
    }
    db.close().await.unwrap();
  }
  // The sole role provisioning entry rejects polluted existing capabilities.
  for flag in ["LOGIN", "INHERIT", "SUPERUSER", "BYPASSRLS", "CREATEROLE", "CREATEDB", "REPLICATION"] {
    fixture
      .bootstrap
      .execute_unprepared(&format!("ALTER ROLE {AUTH_PROVIDER_ACCESS_ROLE} {flag}"))
      .await
      .unwrap();
    let rejected = recorder::migrations::access::create_capability_roles(&fixture.bootstrap).await.is_err();
    fixture
      .bootstrap
      .execute_unprepared(&format!("ALTER ROLE {AUTH_PROVIDER_ACCESS_ROLE} NO{flag}"))
      .await
      .unwrap();
    assert!(rejected, "provisioning must reject capability {flag}");
  }
  fixture.close().await;
}

#[tokio::test]
async fn runtime_startup_rejects_noninherited_capability_memberships_and_unsafe_roles_without_migrations() {
  use std::sync::Arc;

  use konobangu_database::{
    PoolConfig, connect,
    roles::{APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, AUTH_PROVIDER_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE},
  };
  use recorder::{
    auth::{ProviderAuthConfig, connect_identity_db},
    task::{TaskConfig, TaskService},
    test_utils::app::TestingAppContext,
  };

  async fn accepts(fixture: &RlsFixture, role: &'static str) -> bool {
    match role {
      APP_SCOPED_ACCESS_ROLE => {
        let mut config = fixture.bootstrap.config.clone();
        config.uri = fixture.app_uri.clone();
        config.auto_migrate = false;
        match DatabaseService::from_config(config).await {
          Ok(db) => {
            db.as_ref().clone().close().await.unwrap();
            true
          }
          Err(_) => false,
        }
      }
      AUTH_ACCESS_ROLE => match connect_identity_db(&fixture.identity_uri).await {
        Ok(db) => {
          db.close().await.unwrap();
          true
        }
        Err(_) => false,
      },
      TASK_CONTROL_ACCESS_ROLE => {
        let ctx = Arc::new(TestingAppContext::builder().build());
        match TaskService::from_config_and_ctx(
          TaskConfig {
            queue_database_uri: Some(fixture.queue_uri.clone()),
            ..Default::default()
          },
          ctx,
        )
        .await
        {
          Ok(task) => {
            task.close().await.unwrap();
            true
          }
          Err(_) => false,
        }
      }
      AUTH_PROVIDER_ACCESS_ROLE => {
        let ProviderAuthConfig::Bundled(provider) = fixture.bundled_auth("http://127.0.0.1:5001").provider else {
          unreachable!()
        };
        match konobangu_auth_provider::ProviderRuntime::new(*provider, PoolConfig::default()).await {
          Ok(provider) => {
            provider.close().await.unwrap();
            true
          }
          Err(_) => false,
        }
      }
      _ => unreachable!(),
    }
  }

  let fixture = RlsFixture::new().await;
  for role in [APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE, AUTH_PROVIDER_ACCESS_ROLE] {
    assert!(accepts(&fixture, role).await, "valid restricted runtime starts");
  }
  for (source, target, uri) in [
    (APP_SCOPED_ACCESS_ROLE, AUTH_PROVIDER_ACCESS_ROLE, fixture.app_uri.as_str()),
    (AUTH_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE, fixture.identity_uri.as_str()),
    (TASK_CONTROL_ACCESS_ROLE, APP_SCOPED_ACCESS_ROLE, fixture.queue_uri.as_str()),
    (AUTH_PROVIDER_ACCESS_ROLE, AUTH_ACCESS_ROLE, fixture.provider_uri.as_str()),
  ] {
    fixture
      .bootstrap
      .execute_unprepared(&format!("GRANT {target} TO {source} WITH INHERIT FALSE, SET TRUE"))
      .await
      .unwrap();
    let probe = connect(uri, &PoolConfig::default(), Some(source)).await.unwrap();
    let membership = probe
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT pg_has_role(current_user,$1::text,'USAGE') AS inherited, pg_has_role(current_user,$1::text,'MEMBER') AS member",
        [target.into()],
      ))
      .await
      .unwrap()
      .unwrap();
    assert!(!membership.try_get::<bool>("", "inherited").unwrap());
    assert!(membership.try_get::<bool>("", "member").unwrap());
    probe.close().await.unwrap();
    let rejected = !accepts(&fixture, source).await;
    fixture.bootstrap.execute_unprepared(&format!("REVOKE {target} FROM {source}")).await.unwrap();
    assert!(rejected, "SET-capable membership must not escape the {source} boundary");
  }
  for flag in ["LOGIN", "INHERIT", "SUPERUSER", "BYPASSRLS", "CREATEROLE", "CREATEDB", "REPLICATION"] {
    fixture
      .bootstrap
      .execute_unprepared(&format!("ALTER ROLE {APP_SCOPED_ACCESS_ROLE} {flag}"))
      .await
      .unwrap();
    let rejected = !accepts(&fixture, APP_SCOPED_ACCESS_ROLE).await;
    fixture
      .bootstrap
      .execute_unprepared(&format!("ALTER ROLE {APP_SCOPED_ACCESS_ROLE} NO{flag}"))
      .await
      .unwrap();
    assert!(rejected, "auto_migrate=false must still reject {flag}");
  }
  // Database ownership is forbidden even when it owns no application table.
  async fn database_owner(db: &impl ConnectionTrait, owner: &str) {
    let ddl: String = db
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT format('ALTER DATABASE %I OWNER TO %I',current_database(),$1::text) AS ddl",
        [owner.into()],
      ))
      .await
      .unwrap()
      .unwrap()
      .try_get("", "ddl")
      .unwrap();
    db.execute_unprepared(&ddl).await.unwrap();
  }
  let original_owner: String = fixture
    .bootstrap
    .query_one_raw(Statement::from_string(DbBackend::Postgres, "SELECT current_user::text AS owner"))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "owner")
    .unwrap();
  database_owner(&fixture.bootstrap, APP_SCOPED_ACCESS_ROLE).await;
  let owner_rejected = !accepts(&fixture, APP_SCOPED_ACCESS_ROLE).await;
  database_owner(&fixture.bootstrap, &original_owner).await;
  assert!(owner_rejected, "a capability cannot own the application database");

  // Future public tables are covered without extending a table-name checklist.
  fixture
    .bootstrap
    .execute_unprepared("CREATE TABLE capability_guard_fixture (id integer)")
    .await
    .unwrap();
  for role in [AUTH_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE] {
    fixture
      .bootstrap
      .execute_unprepared(&format!("GRANT SELECT ON capability_guard_fixture TO {role}"))
      .await
      .unwrap();
    let rejected = !accepts(&fixture, role).await;
    fixture
      .bootstrap
      .execute_unprepared(&format!("REVOKE SELECT ON capability_guard_fixture FROM {role}"))
      .await
      .unwrap();
    assert!(rejected, "new application tables cannot become visible to {role}");
  }
  fixture.close().await;
}
