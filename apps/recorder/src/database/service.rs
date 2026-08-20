use std::ops::Deref;

use konobangu_database::PoolConfig;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, ExecResult, Iden, QueryResult, Statement, TransactionTrait};
use sea_orm_migration::MigratorTrait;

use super::DatabaseConfig;
use crate::{
  errors::RecorderResult,
  migrations::{
    Migrator,
    defs::{ApalisSchema, ApplicationSchema},
  },
};

pub struct DatabaseService {
  pub config: DatabaseConfig,
  connection: DatabaseConnection,
  #[cfg(feature = "testcontainers")]
  pub container: Option<testcontainers::ContainerAsync<testcontainers_modules::postgres::Postgres>>,
}

impl DatabaseService {
  pub async fn from_app_config(config: &crate::app::AppConfig) -> RecorderResult<Self> {
    let roles = config
      .database
      .auto_migrate
      .then(|| crate::migrations::access::RuntimeLogins::from_config(config))
      .transpose()?;
    Self::runtime(config.database.clone(), roles).await
  }

  pub async fn from_config(config: DatabaseConfig) -> RecorderResult<Self> {
    Self::runtime(config, None).await
  }

  async fn runtime(mut config: DatabaseConfig, roles: Option<crate::migrations::access::RuntimeLogins>) -> RecorderResult<Self> {
    if config.auto_migrate {
      let mut migration = config.clone();
      migration.uri = config.migration_database_uri.clone().unwrap_or_else(|| config.uri.clone());
      let owner = Self::from_migration_config(migration).await?;
      let result = async {
        if let Some(roles) = &roles {
          crate::migrations::access::grant_runtime_memberships(owner.as_ref(), roles).await
        } else {
          crate::migrations::access::grant_membership(
            owner.as_ref(),
            super::roles::APP_SCOPED_ACCESS_ROLE,
            &crate::migrations::access::login(&config.uri)?,
          )
          .await
        }
      }
      .await;
      owner.as_ref().clone().close().await?;
      result?;
      config.auto_migrate = false;
    }
    let me = Self::connect(config, Some(super::roles::APP_SCOPED_ACCESS_ROLE)).await?;
    let validated: RecorderResult<()> = async {
      super::roles::validate_capability_role(me.as_ref(), super::roles::APP_SCOPED_ACCESS_ROLE).await?;
      let row = me
        .query_one_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "SELECT EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname IN ($1,$2,$3,$4) AND \
           pg_has_role(current_user,c.relowner,'MEMBER')) AS owns_tables",
          [
            ApplicationSchema::Public.to_string().into(),
            ApplicationSchema::AuthIdentity.to_string().into(),
            ApalisSchema::Schema.to_string().into(),
            konobangu_auth_provider::provider_schema_name().into(),
          ],
        ))
        .await?
        .ok_or_else(|| DbErr::Custom("Application database role was not found".into()))?;
      if row.try_get::<bool>("", "owns_tables")? {
        return Err(DbErr::Custom("App-scoped pool requires a non-owner NOSUPERUSER NOBYPASSRLS role".into()).into());
      }
      let identity_access = me
        .query_one_raw(Statement::from_sql_and_values(
          DbBackend::Postgres,
          "SELECT has_schema_privilege(current_user,$1,'USAGE') OR has_schema_privilege(current_user,$2,'USAGE') OR \
           has_schema_privilege(current_user,$3,'USAGE') AS identity_access",
          [
            ApplicationSchema::AuthIdentity.to_string().into(),
            ApalisSchema::Schema.to_string().into(),
            konobangu_auth_provider::provider_schema_name().into(),
          ],
        ))
        .await?
        .ok_or_else(|| DbErr::Custom("Identity schema permission check failed".into()))?;
      if identity_access.try_get::<bool>("", "identity_access")? {
        return Err(DbErr::Custom("App-scoped pool must not have auth or task-control access".into()).into());
      }
      Ok(())
    }
    .await;
    if let Err(error) = validated {
      let _ = me.as_ref().clone().close().await;
      return Err(error);
    }
    Ok(me)
  }

  /// Explicit owner-only migration/fixture connection; never inserted into
  /// AppContext.
  pub async fn from_migration_config(config: DatabaseConfig) -> RecorderResult<Self> {
    Self::connect(config, None).await
  }

  async fn connect(config: DatabaseConfig, role: Option<&'static str>) -> RecorderResult<Self> {
    let db_config = config.clone();
    let pool = PoolConfig {
      log_queries: config.enable_logging,
      min_connections: config.min_connections,
      max_connections: config.max_connections,
      connect_timeout_ms: config.connect_timeout,
      idle_timeout_ms: config.idle_timeout,
      acquire_timeout_ms: config.acquire_timeout,
    };
    let db = konobangu_database::connect(&config.uri, &pool, role).await?;

    let me = Self {
      connection: db,
      #[cfg(feature = "testcontainers")]
      container: None,
      config: db_config,
    };

    if config.auto_migrate
      && let Err(error) = me.migrate_up().await
    {
      let _ = me.as_ref().clone().close().await;
      return Err(error);
    }

    Ok(me)
  }

  pub async fn migrate_up(&self) -> RecorderResult<()> {
    let transaction = self.connection.begin().await?;
    transaction
      .query_one_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('konobangu.legacy_oidc_issuer', $1, true)",
        [self.config.legacy_oidc_issuer.clone().unwrap_or_default().into()],
      ))
      .await?;
    Migrator::up(&transaction, None).await?;
    transaction.commit().await?;
    crate::migrations::queue::initialize_queue(&self.connection).await?;
    crate::migrations::access::initialize_access(&self.connection).await?;
    Ok(())
  }

  pub async fn migrate_down(&self) -> RecorderResult<()> {
    Migrator::down(&self.connection, None).await?;
    {
      self.execute_unprepared(r#"DROP SCHEMA IF EXISTS apalis CASCADE"#).await?;
    }
    Ok(())
  }
}

impl Deref for DatabaseService {
  type Target = DatabaseConnection;

  fn deref(&self) -> &Self::Target {
    &self.connection
  }
}

impl AsRef<DatabaseConnection> for DatabaseService {
  fn as_ref(&self) -> &DatabaseConnection {
    &self.connection
  }
}

#[async_trait::async_trait]
impl ConnectionTrait for DatabaseService {
  fn get_database_backend(&self) -> DbBackend {
    self.deref().get_database_backend()
  }

  async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
    self.deref().execute_raw(stmt).await
  }

  async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
    self.deref().execute_unprepared(sql).await
  }

  async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
    self.deref().query_one_raw(stmt).await
  }

  async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
    self.deref().query_all_raw(stmt).await
  }
}
