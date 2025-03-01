use std::{ops::Deref, time::Duration};

use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbBackend,
    DbErr, ExecResult, QueryResult, Statement,
};
use sea_orm_migration::MigratorTrait;

use super::DatabaseConfig;
use crate::{errors::RResult, migrations::Migrator};

pub struct DatabaseService {
    connection: DatabaseConnection,
}

impl DatabaseService {
    pub async fn from_config(config: DatabaseConfig) -> RResult<Self> {
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

        if db.get_database_backend() == DatabaseBackend::Sqlite {
            db.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                "
                PRAGMA foreign_keys = ON;
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA mmap_size = 134217728;
                PRAGMA journal_size_limit = 67108864;
                PRAGMA cache_size = 2000;
                ",
            ))
            .await?;
        }

        if config.auto_migrate {
            Migrator::up(&db, None).await?;
        }

        Ok(Self { connection: db })
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
