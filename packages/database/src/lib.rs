//! Shared connection and migration primitives, independent of application
//! services.
pub mod migration;
pub mod roles;

use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PoolConfig {
  pub log_queries: bool,
  pub min_connections: u32,
  pub max_connections: u32,
  pub connect_timeout_ms: u64,
  pub idle_timeout_ms: u64,
  pub acquire_timeout_ms: Option<u64>,
}

impl Default for PoolConfig {
  fn default() -> Self {
    Self {
      log_queries: false,
      min_connections: 1,
      max_connections: 5,
      connect_timeout_ms: 5000,
      idle_timeout_ms: 60000,
      acquire_timeout_ms: None,
    }
  }
}

/// Credentials remain at the composition boundary; connection errors never
/// include the URL or driver error text.
pub async fn connect(database_url: &str, pool: &PoolConfig, role: Option<&'static str>) -> Result<DatabaseConnection, DbErr> {
  if pool.max_connections == 0 || pool.min_connections > pool.max_connections {
    return Err(DbErr::Custom(
      "Database pool requires positive capacity and min_connections <= max_connections".into(),
    ));
  }
  let mut options = ConnectOptions::new(database_url);
  options
    .max_connections(pool.max_connections)
    .min_connections(pool.min_connections)
    .connect_timeout(Duration::from_millis(pool.connect_timeout_ms))
    .idle_timeout(Duration::from_millis(pool.idle_timeout_ms))
    .sqlx_logging(pool.log_queries);
  if let Some(timeout) = pool.acquire_timeout_ms {
    options.acquire_timeout(Duration::from_millis(timeout));
  }
  if let Some(role) = role {
    roles::restrict_pool(&mut options, role);
  }
  Database::connect(options)
    .await
    .map_err(|_| DbErr::Custom("Database connection failed (details redacted)".into()))
}
