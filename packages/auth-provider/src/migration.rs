use sea_orm::{ConnectionTrait, DbErr, EntityName};

use crate::schema::{Entity, State, schema_name, table};

pub async fn apply_migrations(owner: &impl ConnectionTrait) -> Result<(), DbErr> {
  use sea_orm_migration::prelude::*;
  owner.execute_unprepared(&format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema_name())).await?;
  owner
    .execute(
      &Table::create()
        .table(Entity.table_ref())
        .if_not_exists()
        .col(ColumnDef::new(State::Hash).text().primary_key())
        .col(ColumnDef::new(State::Kind).text().not_null())
        .col(ColumnDef::new(State::Data).json_binary().not_null())
        .col(ColumnDef::new(State::Family).text())
        .col(ColumnDef::new(State::ExpiresAt).big_integer().not_null())
        .col(ColumnDef::new(State::Consumed).boolean().not_null().default(false))
        .to_owned(),
    )
    .await?;
  owner
    .execute(
      &Index::create()
        .name("protocol_state_expiry")
        .table(Entity.table_ref())
        .col(State::ExpiresAt)
        .if_not_exists()
        .to_owned(),
    )
    .await?;
  Ok(())
}
pub async fn revert_migrations(owner: &impl ConnectionTrait) -> Result<(), DbErr> {
  owner
    .execute_unprepared(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema_name()))
    .await?;
  Ok(())
}
pub async fn grant_access(owner: &impl ConnectionTrait) -> Result<(), DbErr> {
  let role = konobangu_database::roles::AUTH_PROVIDER_ACCESS_ROLE;
  owner
    .execute_unprepared(&format!(
      "GRANT USAGE ON SCHEMA \"{}\" TO {role}; GRANT SELECT,INSERT,UPDATE,DELETE ON {} TO {role}",
      schema_name(),
      table()
    ))
    .await?;
  Ok(())
}
