use std::{ops::Deref, time::Duration};

use apalis_sql::postgres::PostgresStorage;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, ExecResult,
    QueryResult, Statement,
};
use sea_orm_migration::MigratorTrait;

use super::DatabaseConfig;
use crate::{errors::RecorderResult, migrations::Migrator};

pub struct DatabaseService {
    pub config: DatabaseConfig,
    connection: DatabaseConnection,
    #[cfg(feature = "testcontainers")]
    pub container:
        Option<testcontainers::ContainerAsync<testcontainers_modules::postgres::Postgres>>,
}

impl DatabaseService {
    pub async fn from_config(config: DatabaseConfig) -> RecorderResult<Self> {
        let db_config = config.clone();
        let mut opt = ConnectOptions::new(&config.uri);
        opt.max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(Duration::from_millis(config.connect_timeout))
            .idle_timeout(Duration::from_millis(config.idle_timeout))
            .sqlx_logging(config.enable_logging);

        if let Some(acquire_timeout) = config.acquire_timeout {
            opt.acquire_timeout(Duration::from_millis(acquire_timeout));
        }

        let db = Database::connect(opt).await?;

        // only support postgres for now
        // if db.get_database_backend() == DatabaseBackend::Sqlite {
        //     db.execute(Statement::from_string(
        //         DatabaseBackend::Sqlite,
        //         "
        //         PRAGMA foreign_keys = ON;
        //         PRAGMA journal_mode = WAL;
        //         PRAGMA synchronous = NORMAL;
        //         PRAGMA mmap_size = 134217728;
        //         PRAGMA journal_size_limit = 67108864;
        //         PRAGMA cache_size = 2000;
        //         ",
        //     ))
        //     .await?;
        // }

        let me = Self {
            connection: db,
            #[cfg(feature = "testcontainers")]
            container: None,
            config: db_config,
        };

        if config.auto_migrate {
            me.migrate_up().await?;
        }

        Ok(me)
    }

    pub async fn migrate_up(&self) -> RecorderResult<()> {
        {
            let pool = &self.get_postgres_connection_pool();
            PostgresStorage::setup(pool).await?;
        }
        Migrator::up(&self.connection, None).await?;
        Ok(())
    }

    pub async fn migrate_down(&self) -> RecorderResult<()> {
        Migrator::down(&self.connection, None).await?;
        {
            self.execute_unprepared(r#"DROP SCHEMA IF EXISTS apalis CASCADE"#)
                .await?;
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

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.deref().execute(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.deref().execute_unprepared(sql).await
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.deref().query_one(stmt).await
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.deref().query_all(stmt).await
    }

    fn support_returning(&self) -> bool {
        self.deref().support_returning()
    }

    fn is_mock_connection(&self) -> bool {
        self.deref().is_mock_connection()
    }
}
