use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, TransactionTrait};
use serde::{Deserialize, Serialize};

pub use super::entities::subscribers::*;

pub const ROOT_SUBSCRIBER: &str = "konobangu";

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscriberIdParams {
    pub id: String,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            let mut this = self;
            this.pid = ActiveValue::Set(Uuid::new_v4().to_string());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

impl Model {
    /// finds a user by the provided pid
    ///
    /// # Errors
    ///
    /// When could not find user  or DB query error
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &str) -> ModelResult<Self> {
        let parse_uuid = Uuid::parse_str(pid).map_err(|e| ModelError::Any(e.into()))?;
        let subscriber = Entity::find()
            .filter(Column::Pid.eq(parse_uuid))
            .one(db)
            .await?;
        subscriber.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_root(db: &DatabaseConnection) -> ModelResult<Self> {
        Self::find_by_pid(db, ROOT_SUBSCRIBER).await
    }

    /// Asynchronously creates a user with a password and saves it to the
    /// database.
    ///
    /// # Errors
    ///
    /// When could not save the user into the DB
    pub async fn create_root(db: &DatabaseConnection) -> ModelResult<Self> {
        let txn = db.begin().await?;

        let user = ActiveModel {
            display_name: ActiveValue::set(ROOT_SUBSCRIBER.to_string()),
            pid: ActiveValue::set(ROOT_SUBSCRIBER.to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }
}
