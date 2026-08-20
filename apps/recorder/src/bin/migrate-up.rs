use recorder::{
  app::AppBuilder,
  database::DatabaseService,
  errors::RecorderResult,
  migrations::access::{RuntimeLogins, grant_runtime_memberships},
};

#[tokio::main]
async fn main() -> RecorderResult<()> {
  let builder = AppBuilder::from_main_cli(None).await?;
  builder.load_env().await?;
  let app_config = builder.load_config().await?;
  let roles = RuntimeLogins::from_config(&app_config)?;
  let mut config = app_config.database;
  if let Some(uri) = config.migration_database_uri.take() {
    config.uri = uri;
  }
  config.auto_migrate = false;
  let database = DatabaseService::from_migration_config(config).await?;
  let result: RecorderResult<()> = async {
    database.migrate_up().await?;
    grant_runtime_memberships(database.as_ref(), &roles).await?;
    Ok(())
  }
  .await;
  database.as_ref().clone().close().await?;
  result?;
  Ok(())
}
