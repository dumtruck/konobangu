//! The delivery queue schema is owned by the upstream storage migrator.
use sea_orm::{DatabaseConnection, DbErr};

pub async fn initialize_queue(db: &DatabaseConnection) -> Result<(), DbErr> {
  // Explicitly apply the published sqlx.toml settings to the public migrator.
  let mut migrator = apalis_postgres::PostgresStorage::<()>::migrations();
  migrator.dangerous_set_table_name("apalis._sqlx_migrations");
  migrator.create_schema("apalis");
  migrator
    .run(db.get_postgres_connection_pool())
    .await
    .map_err(|e| DbErr::Migration(e.to_string()))?;
  Ok(())
}
