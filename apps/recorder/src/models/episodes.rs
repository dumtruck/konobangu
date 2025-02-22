use std::sync::Arc;

use async_trait::async_trait;
use loco_rs::app::AppContext;
use sea_orm::{ActiveValue, FromJsonQueryResult, entity::prelude::*, sea_query::OnConflict};
use serde::{Deserialize, Serialize};

use super::{bangumi, query::InsertManyReturningExt, subscription_episode};
use crate::{
    app::AppContextExt,
    extract::{
        mikan::{MikanEpisodeMeta, build_mikan_episode_homepage},
        rawname::parse_episode_meta_from_raw_name,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, Default)]
pub struct EpisodeExtra {
    pub name_zh: Option<String>,
    pub s_name_zh: Option<String>,
    pub name_en: Option<String>,
    pub s_name_en: Option<String>,
    pub name_jp: Option<String>,
    pub s_name_jp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub mikan_episode_id: Option<String>,
    pub raw_name: String,
    pub display_name: String,
    pub bangumi_id: i32,
    pub subscriber_id: i32,
    pub save_path: Option<String>,
    pub resolution: Option<String>,
    pub season: i32,
    pub season_raw: Option<String>,
    pub fansub: Option<String>,
    pub poster_link: Option<String>,
    pub episode_index: i32,
    pub homepage: Option<String>,
    pub subtitle: Option<String>,
    #[sea_orm(default = "false")]
    pub deleted: bool,
    pub source: Option<String>,
    pub extra: EpisodeExtra,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Subscriber,
    #[sea_orm(
        belongs_to = "super::bangumi::Entity",
        from = "Column::BangumiId",
        to = "super::bangumi::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Bangumi,
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(has_one = "super::downloads::Entity")]
    Download,
    #[sea_orm(has_many = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

impl Related<super::downloads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Download.def()
    }
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::subscription_episode::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionEpisode.def()
    }
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_episode::Relation::Subscription.def()
    }

    fn via() -> Option<RelationDef> {
        Some(Relation::Subscription.def())
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::downloads::Entity")]
    Subscription,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Download,
    #[sea_orm(entity = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpsiodeCreation {
    pub episode: MikanEpisodeMeta,
    pub bangumi: Arc<bangumi::Model>,
}

impl Model {
    pub async fn add_episodes(
        ctx: &AppContext,
        subscriber_id: i32,
        subscription_id: i32,
        creations: impl IntoIterator<Item = MikanEpsiodeCreation>,
    ) -> color_eyre::eyre::Result<()> {
        let db = &ctx.db;
        let new_episode_active_modes = creations
            .into_iter()
            .map(|cr| ActiveModel::from_mikan_episode_meta(ctx, cr))
            .inspect(|result| {
                if let Err(e) = result {
                    tracing::warn!("Failed to create episode: {:?}", e);
                }
            })
            .flatten();

        let inserted_episodes = Entity::insert_many(new_episode_active_modes)
            .on_conflict(
                OnConflict::columns([Column::BangumiId, Column::MikanEpisodeId])
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning_columns(db, [Column::Id])
            .await?
            .into_iter()
            .flat_map(|r| r.try_get_many_by_index::<i32>());

        let insert_subscription_episode_links = inserted_episodes.into_iter().map(|episode_id| {
            subscription_episode::ActiveModel::from_subscription_and_episode(
                subscriber_id,
                subscription_id,
                episode_id,
            )
        });

        subscription_episode::Entity::insert_many(insert_subscription_episode_links)
            .on_conflict(
                OnConflict::columns([
                    subscription_episode::Column::SubscriptionId,
                    subscription_episode::Column::EpisodeId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await?;

        Ok(())
    }
}

impl ActiveModel {
    pub fn from_mikan_episode_meta(
        ctx: &AppContext,
        creation: MikanEpsiodeCreation,
    ) -> color_eyre::eyre::Result<Self> {
        let item = creation.episode;
        let bgm = creation.bangumi;
        let raw_meta = parse_episode_meta_from_raw_name(&item.episode_title)
            .inspect_err(|e| {
                tracing::warn!("Failed to parse episode meta: {:?}", e);
            })
            .ok()
            .unwrap_or_default();
        let homepage = build_mikan_episode_homepage(
            ctx.get_mikan_client().base_url(),
            &item.mikan_episode_id,
        )?;

        Ok(Self {
            mikan_episode_id: ActiveValue::Set(Some(item.mikan_episode_id)),
            raw_name: ActiveValue::Set(item.episode_title.clone()),
            display_name: ActiveValue::Set(item.episode_title.clone()),
            bangumi_id: ActiveValue::Set(bgm.id),
            subscriber_id: ActiveValue::Set(bgm.subscriber_id),
            resolution: ActiveValue::Set(raw_meta.resolution),
            season: ActiveValue::Set(if raw_meta.season > 0 {
                raw_meta.season
            } else {
                bgm.season
            }),
            season_raw: ActiveValue::Set(raw_meta.season_raw.or_else(|| bgm.season_raw.clone())),
            fansub: ActiveValue::Set(raw_meta.fansub.or_else(|| bgm.fansub.clone())),
            poster_link: ActiveValue::Set(bgm.poster_link.clone()),
            episode_index: ActiveValue::Set(raw_meta.episode_index),
            homepage: ActiveValue::Set(Some(homepage.to_string())),
            subtitle: ActiveValue::Set(raw_meta.subtitle),
            source: ActiveValue::Set(raw_meta.source),
            extra: ActiveValue::Set(EpisodeExtra {
                name_zh: raw_meta.name_zh,
                name_en: raw_meta.name_en,
                name_jp: raw_meta.name_jp,
                s_name_en: raw_meta.name_en_no_season,
                s_name_jp: raw_meta.name_jp_no_season,
                s_name_zh: raw_meta.name_zh_no_season,
            }),
            ..Default::default()
        })
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
