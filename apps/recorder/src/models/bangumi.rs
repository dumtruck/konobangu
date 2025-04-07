use async_graphql::SimpleObject;
use async_trait::async_trait;
use sea_orm::{ActiveValue, FromJsonQueryResult, entity::prelude::*, sea_query::OnConflict};
use serde::{Deserialize, Serialize};

use super::subscription_bangumi;
use crate::{app::AppContextTrait, errors::RecorderResult};

#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, SimpleObject,
)]
pub struct BangumiFilter {
    pub name: Option<Vec<String>>,
    pub group: Option<Vec<String>>,
}

#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, SimpleObject,
)]
pub struct BangumiExtra {
    pub name_zh: Option<String>,
    pub s_name_zh: Option<String>,
    pub name_en: Option<String>,
    pub s_name_en: Option<String>,
    pub name_jp: Option<String>,
    pub s_name_jp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, SimpleObject)]
#[sea_orm(table_name = "bangumi")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mikan_bangumi_id: Option<String>,
    pub subscriber_id: i32,
    pub display_name: String,
    pub raw_name: String,
    pub season: i32,
    pub season_raw: Option<String>,
    pub fansub: Option<String>,
    pub mikan_fansub_id: Option<String>,
    pub filter: Option<BangumiFilter>,
    pub rss_link: Option<String>,
    pub poster_link: Option<String>,
    pub save_path: Option<String>,
    #[sea_orm(default = "false")]
    pub deleted: bool,
    pub homepage: Option<String>,
    pub extra: Option<BangumiExtra>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Subscriber,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episode,
    #[sea_orm(has_many = "super::subscription_bangumi::Entity")]
    SubscriptionBangumi,
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Episode.def()
    }
}

impl Related<super::subscription_bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionBangumi.def()
    }
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_bangumi::Relation::Subscription.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::subscription_bangumi::Relation::Bangumi.def().rev())
    }
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::episodes::Entity")]
    Episode,
    #[sea_orm(entity = "super::subscription_bangumi::Entity")]
    SubscriptionBangumi,
}

impl Model {
    pub async fn get_or_insert_from_mikan<F>(
        ctx: &dyn AppContextTrait,
        subscriber_id: i32,
        subscription_id: i32,
        mikan_bangumi_id: String,
        mikan_fansub_id: String,
        f: F,
    ) -> RecorderResult<Model>
    where
        F: AsyncFnOnce(&mut ActiveModel) -> RecorderResult<()>,
    {
        let db = ctx.db();
        if let Some(existed) = Entity::find()
            .filter(
                Column::MikanBangumiId
                    .eq(Some(mikan_bangumi_id.clone()))
                    .and(Column::MikanFansubId.eq(Some(mikan_fansub_id.clone()))),
            )
            .one(db)
            .await?
        {
            Ok(existed)
        } else {
            let mut bgm = ActiveModel {
                mikan_bangumi_id: ActiveValue::Set(Some(mikan_bangumi_id)),
                mikan_fansub_id: ActiveValue::Set(Some(mikan_fansub_id)),
                subscriber_id: ActiveValue::Set(subscriber_id),
                ..Default::default()
            };
            f(&mut bgm).await?;
            let bgm = Entity::insert(bgm)
                .on_conflict(
                    OnConflict::columns([
                        Column::MikanBangumiId,
                        Column::MikanFansubId,
                        Column::SubscriberId,
                    ])
                    .update_columns([
                        Column::RawName,
                        Column::Extra,
                        Column::Fansub,
                        Column::PosterLink,
                        Column::Season,
                        Column::SeasonRaw,
                    ])
                    .to_owned(),
                )
                .exec_with_returning(db)
                .await?;
            subscription_bangumi::Entity::insert(subscription_bangumi::ActiveModel {
                subscription_id: ActiveValue::Set(subscription_id),
                bangumi_id: ActiveValue::Set(bgm.id),
                ..Default::default()
            })
            .on_conflict_do_nothing()
            .exec(db)
            .await?;
            Ok(bgm)
        }
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
