use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{entity::prelude::*, ActiveValue, FromJsonQueryResult, TransactionTrait};
use serde::{Deserialize, Serialize};

use super::bangumi::BangumiRenameMethod;

pub const ROOT_SUBSCRIBER_NAME: &str = "konobangu";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct SubscribeBangumiConfig {
    pub leading_fansub_tag: bool,
    pub complete_history_episodes: bool,
    pub rename_method: BangumiRenameMethod,
    pub remove_bad_torrent: bool,
}

impl Default for SubscribeBangumiConfig {
    fn default() -> Self {
        Self {
            leading_fansub_tag: false,
            complete_history_episodes: false,
            rename_method: BangumiRenameMethod::Pn,
            remove_bad_torrent: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscribers")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pid: String,
    pub display_name: String,
    pub downloader_id: Option<i32>,
    pub bangumi_conf: Option<SubscribeBangumiConfig>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(
        belongs_to = "super::downloaders::Entity",
        from = "Column::DownloaderId",
        to = "super::downloaders::Column::Id"
    )]
    Downloader,
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::downloaders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Downloader.def()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscriberIdParams {
    pub pid: String,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            let mut this = self;
            if this.pid.is_not_set() {
                this.pid = ActiveValue::Set(Uuid::new_v4().to_string());
            };
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

impl Model {
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &str) -> ModelResult<Self> {
        let subscriber = Entity::find().filter(Column::Pid.eq(pid)).one(db).await?;
        subscriber.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> ModelResult<Self> {
        let subscriber = Entity::find().filter(Column::Id.eq(id)).one(db).await?;
        subscriber.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_root(db: &DatabaseConnection) -> ModelResult<Self> {
        Self::find_by_pid(db, ROOT_SUBSCRIBER_NAME).await
    }

    pub async fn create_root(db: &DatabaseConnection) -> ModelResult<Self> {
        let txn = db.begin().await?;

        let user = ActiveModel {
            display_name: ActiveValue::set(ROOT_SUBSCRIBER_NAME.to_string()),
            pid: ActiveValue::set(ROOT_SUBSCRIBER_NAME.to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }
}
