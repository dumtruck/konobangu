use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, TransactionTrait};
use serde::{Deserialize, Serialize};

pub use super::_entities::subscribers::{self, ActiveModel, Entity, Model};

pub const ROOT_SUBSCRIBER: &str = "konobangu";

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscriberIdParams {
    pub id: String,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::subscribers::ActiveModel {
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

impl super::_entities::subscribers::Model {
    /// finds a user by the provided pid
    ///
    /// # Errors
    ///
    /// When could not find user  or DB query error
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &str) -> ModelResult<Self> {
        let parse_uuid = Uuid::parse_str(pid).map_err(|e| ModelError::Any(e.into()))?;
        let subscriber = subscribers::Entity::find()
            .filter(subscribers::Column::Pid.eq(parse_uuid))
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

        let user = subscribers::ActiveModel {
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
