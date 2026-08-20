use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr};
use sqlx::Row;

pub const APP_SCOPED_ACCESS_ROLE: &str = "konobangu_app_scoped_access";
pub const AUTH_ACCESS_ROLE: &str = "konobangu_auth_access";
pub const TASK_CONTROL_ACCESS_ROLE: &str = "konobangu_task_control_access";
pub const AUTH_PROVIDER_ACCESS_ROLE: &str = "konobangu_auth_provider_access";
pub const CAPABILITY_ROLES: [&str; 4] = [APP_SCOPED_ACCESS_ROLE, AUTH_ACCESS_ROLE, TASK_CONTROL_ACCESS_ROLE, AUTH_PROVIDER_ACCESS_ROLE];

/// NOINHERIT does not prevent SET ROLE through membership. Validate the actual
/// capability even when owner migrations are intentionally disabled.
pub async fn validate_capability_role(db: &DatabaseConnection, role: &'static str) -> Result<(), DbErr> {
  if !CAPABILITY_ROLES.contains(&role) {
    return Err(DbErr::Custom("Unknown database capability role".into()));
  }
  let unavailable = || DbErr::Custom("Database capability validation failed (details redacted)".into());
  let row = sqlx::query(
    "SELECT current_user::text AS role_name, (r.rolcanlogin OR r.rolinherit OR r.rolsuper OR r.rolbypassrls OR r.rolcreaterole OR r.rolcreatedb OR \
     r.rolreplication) AS unsafe_role,
     EXISTS(SELECT 1 FROM pg_roles candidate WHERE candidate.rolname=ANY($1::text[]) AND candidate.rolname<>current_user AND \
     pg_has_role(current_user,candidate.oid,'MEMBER')) AS cross_capabilities,
     EXISTS(SELECT 1 FROM pg_database database WHERE database.datname=current_database() AND pg_has_role(current_user,database.datdba,'MEMBER')) AS \
     database_owner
     FROM pg_roles r WHERE r.rolname=current_user",
  )
  .bind(CAPABILITY_ROLES.map(str::to_owned).to_vec())
  .fetch_one(db.get_postgres_connection_pool())
  .await
  .map_err(|_| unavailable())?;
  if row.try_get::<String, _>("role_name").map_err(|_| unavailable())? != role
    || row.try_get::<bool, _>("unsafe_role").map_err(|_| unavailable())?
    || row.try_get::<bool, _>("cross_capabilities").map_err(|_| unavailable())?
    || row.try_get::<bool, _>("database_owner").map_err(|_| unavailable())?
  {
    return Err(DbErr::Custom(
      "Capability role must be isolated, non-owner and without login, inheritance or administrative privileges".into(),
    ));
  }
  Ok(())
}

/// Library queries acquire their own connections. Every physical connection
/// therefore has a restricted baseline, restored before each pool checkout.
pub fn restrict_pool(options: &mut ConnectOptions, role: &'static str) {
  assert!(CAPABILITY_ROLES.contains(&role));
  options.map_sqlx_postgres_pool_opts(move |pool| {
    pool
      .after_connect(move |connection, _| {
        Box::pin(async move {
          sqlx::query("SELECT set_config('role', $1, false)").bind(role).execute(connection).await?;
          Ok(())
        })
      })
      .before_acquire(move |connection, _| {
        Box::pin(async move {
          sqlx::query("SELECT set_config('role', $1, false)").bind(role).execute(connection).await?;
          Ok(true)
        })
      })
  });
}

/// Role and subscriber scope are transaction-local; completion or cancellation
/// returns to the pool's restricted baseline.
pub async fn set_local_role(db: &DatabaseTransaction, role: &'static str) -> Result<(), DbErr> {
  assert!(CAPABILITY_ROLES.contains(&role));
  db.execute_unprepared(&format!("SET LOCAL ROLE {role}")).await?;
  Ok(())
}
