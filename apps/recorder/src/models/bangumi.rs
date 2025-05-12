use std::sync::Arc;

use async_graphql::SimpleObject;
use async_trait::async_trait;
use sea_orm::{
    ActiveValue, FromJsonQueryResult, FromQueryResult, IntoSimpleExpr, JoinType, QuerySelect,
    entity::prelude::*,
    sea_query::{IntoCondition, OnConflict},
};
use serde::{Deserialize, Serialize};

use super::subscription_bangumi;
use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::{
        mikan::{
            MikanBangumiHash, MikanBangumiMeta, build_mikan_bangumi_subscription_rss_url,
            scrape_mikan_poster_meta_from_image_url,
        },
        rawname::parse_episode_meta_from_raw_name,
    },
};

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
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
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

impl ActiveModel {
    #[tracing::instrument(err, skip_all, fields(mikan_bangumi_id = %meta.mikan_bangumi_id, mikan_fansub_id = %meta.mikan_fansub_id, subscriber_id = %subscriber_id))]
    pub async fn from_mikan_bangumi_meta(
        ctx: &dyn AppContextTrait,
        meta: MikanBangumiMeta,
        subscriber_id: i32,
        _subscription_id: i32,
    ) -> RecorderResult<Self> {
        let mikan_client = ctx.mikan();
        let storage_service = ctx.storage();
        let mikan_base_url = mikan_client.base_url();

        let raw_meta = parse_episode_meta_from_raw_name(&meta.bangumi_title)?;

        let rss_url = build_mikan_bangumi_subscription_rss_url(
            mikan_base_url.clone(),
            &meta.mikan_bangumi_id,
            Some(&meta.mikan_fansub_id),
        );

        let poster_link = if let Some(origin_poster_src) = meta.origin_poster_src {
            let poster_meta = scrape_mikan_poster_meta_from_image_url(
                mikan_client,
                storage_service,
                origin_poster_src,
                subscriber_id,
            )
            .await?;
            poster_meta.poster_src
        } else {
            None
        };

        Ok(Self {
            mikan_bangumi_id: ActiveValue::Set(Some(meta.mikan_bangumi_id)),
            mikan_fansub_id: ActiveValue::Set(Some(meta.mikan_fansub_id)),
            subscriber_id: ActiveValue::Set(subscriber_id),
            display_name: ActiveValue::Set(meta.bangumi_title.clone()),
            raw_name: ActiveValue::Set(meta.bangumi_title),
            season: ActiveValue::Set(raw_meta.season),
            season_raw: ActiveValue::Set(raw_meta.season_raw),
            fansub: ActiveValue::Set(Some(meta.fansub)),
            poster_link: ActiveValue::Set(poster_link),
            homepage: ActiveValue::Set(Some(meta.homepage.to_string())),
            rss_link: ActiveValue::Set(Some(rss_url.to_string())),
            ..Default::default()
        })
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

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

    pub async fn get_existed_mikan_bangumi_list(
        ctx: &dyn AppContextTrait,
        hashes: impl Iterator<Item = MikanBangumiHash>,
        subscriber_id: i32,
        _subscription_id: i32,
    ) -> RecorderResult<impl Iterator<Item = (i32, MikanBangumiHash)>> {
        Ok(Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::MikanBangumiId)
            .column(Column::MikanFansubId)
            .filter(
                Expr::tuple([
                    Column::MikanBangumiId.into_simple_expr(),
                    Column::MikanFansubId.into_simple_expr(),
                    Column::SubscriberId.into_simple_expr(),
                ])
                .in_tuples(hashes.map(|hash| {
                    (
                        hash.mikan_bangumi_id.clone(),
                        hash.mikan_fansub_id.clone(),
                        subscriber_id,
                    )
                })),
            )
            .into_tuple::<(i32, String, String)>()
            .all(ctx.db())
            .await?
            .into_iter()
            .map(|(bangumi_id, mikan_bangumi_id, mikan_fansub_id)| {
                (
                    bangumi_id,
                    MikanBangumiHash {
                        mikan_bangumi_id,
                        mikan_fansub_id,
                    },
                )
            }))
    }
}
