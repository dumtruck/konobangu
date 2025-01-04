use async_trait::async_trait;
use loco_rs::{
    app::AppContext,
    model::{ModelError, ModelResult},
};
use sea_orm::{entity::prelude::*, ActiveValue, FromJsonQueryResult, TransactionTrait};
use serde::{Deserialize, Serialize};

pub const SEED_SUBSCRIBER: &str = "konobangu";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct SubscriberBangumiConfig {
    pub leading_group_tag: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscribers")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: String,
    pub display_name: String,
    pub bangumi_conf: Option<SubscriberBangumiConfig>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(has_many = "super::downloaders::Entity")]
    Downloader,
    #[sea_orm(has_many = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episode,
    #[sea_orm(has_many = "super::auth::Entity")]
    Auth,
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

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Episode.def()
    }
}

impl Related<super::auth::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Auth.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(entity = "super::downloaders::Entity")]
    Downloader,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(entity = "super::episodes::Entity")]
    Episode,
    #[sea_orm(entity = "super::auth::Entity")]
    Auth,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscriberIdParams {
    pub id: String,
}

#[async_trait]
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
    pub async fn find_by_pid(ctx: &AppContext, pid: &str) -> ModelResult<Self> {
        let db = &ctx.db;
        let parse_uuid = Uuid::parse_str(pid).map_err(|e| ModelError::Any(e.into()))?;
        let subscriber = Entity::find()
            .filter(Column::Pid.eq(parse_uuid))
            .one(db)
            .await?;
        subscriber.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_by_id(ctx: &AppContext, id: i32) -> ModelResult<Self> {
        let db = &ctx.db;

        let subscriber = Entity::find_by_id(id).one(db).await?;
        subscriber.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_pid_by_id_with_cache(ctx: &AppContext, id: i32) -> eyre::Result<String> {
        let db = &ctx.db;
        let cache = &ctx.cache;
        let pid = cache
            .get_or_insert(&format!("subscriber-id2pid::{}", id), async {
                let subscriber = Entity::find_by_id(id)
                    .one(db)
                    .await?
                    .ok_or_else(|| loco_rs::Error::string(&format!("No such pid for id {}", id)))?;
                Ok(subscriber.pid)
            })
            .await?;
        Ok(pid)
    }

    pub async fn find_root(ctx: &AppContext) -> ModelResult<Self> {
        Self::find_by_pid(ctx, SEED_SUBSCRIBER).await
    }

    /// Asynchronously creates a user with a password and saves it to the
    /// database.
    ///
    /// # Errors
    ///
    /// When could not save the user into the DB
    pub async fn create_root(ctx: &AppContext) -> ModelResult<Self> {
        let db = &ctx.db;
        let txn = db.begin().await?;

        let user = ActiveModel {
            display_name: ActiveValue::set(SEED_SUBSCRIBER.to_string()),
            pid: ActiveValue::set(SEED_SUBSCRIBER.to_string()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }
}
