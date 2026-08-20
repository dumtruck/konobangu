//! Restricted identity database connection and startup permission guards.
//!
//! Identity mapping is business data, not an authentication session store.
use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityName, Iden};

use crate::{
  auth::AuthError,
  database::roles::{AUTH_ACCESS_ROLE, restrict_pool, validate_capability_role},
  migrations::defs::{ApalisSchema, ApplicationSchema},
  models::auth,
};

/// Connect an identity pool bounded at three connections and verify its
/// isolation.
pub async fn connect_identity_db(database_url: &str) -> Result<DatabaseConnection, AuthError> {
  let invalid = |message: &str| AuthError::Configuration { message: message.into() };
  let mut options = ConnectOptions::new(database_url);
  options.sqlx_logging(false).max_connections(3).min_connections(1);
  restrict_pool(&mut options, AUTH_ACCESS_ROLE);
  let identity_db = Database::connect(options).await.map_err(|_| AuthError::IdentityStorage)?;
  let guarded = async {
    validate_capability_role(&identity_db, AUTH_ACCESS_ROLE)
      .await
      .map_err(|_| invalid("identity capability is not isolated"))?;
    let pool = identity_db.get_postgres_connection_pool();
    let owns_tables: bool = sqlx::query_scalar(
      "SELECT EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1 AND c.relname=$2 AND \
       pg_has_role(current_user,c.relowner,'MEMBER'))",
    )
    .bind(auth::Entity.schema_name().ok_or(AuthError::IdentityStorage)?)
    .bind(auth::Entity.table_name())
    .fetch_one(pool)
    .await
    .map_err(|_| AuthError::IdentityStorage)?;
    if owns_tables {
      return Err(invalid("identity pool must not own identity tables"));
    }
    // Column-limited subscriber initialization does not confer table-level
    // access. Catalog checks automatically include future business tables.
    let private_access: bool = sqlx::query_scalar(
      "SELECT EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname=$1 AND c.relkind IN ('r','p','v','m','f') AND \
       has_table_privilege(current_user, c.oid, 'SELECT,INSERT,UPDATE,DELETE')) OR has_schema_privilege(current_user,$2,'USAGE') OR \
       has_schema_privilege(current_user,$3,'USAGE')",
    )
    .bind(ApplicationSchema::Public.to_string())
    .bind(konobangu_auth_provider::provider_schema_name())
    .bind(ApalisSchema::Schema.to_string())
    .fetch_one(pool)
    .await
    .map_err(|_| AuthError::IdentityStorage)?;
    if private_access {
      return Err(invalid("identity pool must not have private application or task-control privileges"));
    }
    Ok(())
  }
  .await;
  if let Err(error) = guarded {
    let _ = identity_db.close().await;
    return Err(error);
  }
  Ok(identity_db)
}
