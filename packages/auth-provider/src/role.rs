use konobangu_database::roles::{AUTH_PROVIDER_ACCESS_ROLE, validate_capability_role};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

use crate::ProviderError;

/// Shared role invariants apply even when owner migrations are disabled.
/// Catalog checks then establish this domain's schema and table boundaries.
pub(crate) async fn validate(db: &DatabaseConnection) -> Result<(), ProviderError> {
  validate_capability_role(db, AUTH_PROVIDER_ACCESS_ROLE)
    .await
    .map_err(|_| ProviderError::Configuration)?;
  let query = Statement::from_sql_and_values(
    DbBackend::Postgres,
    r#"
SELECT has_table_privilege(current_user,$1,'SELECT') AND has_table_privilege(current_user,$1,'INSERT') AND has_table_privilege(current_user,$1,'UPDATE') AND has_table_privilege(current_user,$1,'DELETE') AS usable_state,
       EXISTS(SELECT 1 FROM pg_namespace n WHERE n.nspname IN ('auth_identity','apalis') AND has_schema_privilege(current_user,n.oid,'USAGE')) AS other_schema,
       EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname IN ('public','auth_identity','apalis') AND c.relkind IN ('r','p','v','m') AND has_table_privilege(current_user,c.oid,'SELECT,INSERT,UPDATE,DELETE')) AS business_access,
       EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$2 AND pg_has_role(current_user,c.relowner,'MEMBER')) AS owns_state
"#,
    vec![crate::schema::table().into(), crate::schema::schema_name().into()],
  );
  let row = db
    .query_one_raw(query)
    .await
    .map_err(|_| ProviderError::Configuration)?
    .ok_or(ProviderError::Configuration)?;
  for flag in ["other_schema", "business_access", "owns_state"] {
    if row.try_get::<bool>("", flag).map_err(|_| ProviderError::Configuration)? {
      return Err(ProviderError::Configuration);
    }
  }
  if !row.try_get::<bool>("", "usable_state").map_err(|_| ProviderError::Configuration)? {
    return Err(ProviderError::Configuration);
  }
  // Also prove usable state permissions now, rather than failing at first
  // login.
  db.query_one_raw(Statement::from_string(
    DbBackend::Postgres,
    format!("SELECT 1 FROM {} LIMIT 0", crate::schema::table()),
  ))
  .await
  .map_err(|_| ProviderError::Configuration)?;
  Ok(())
}
