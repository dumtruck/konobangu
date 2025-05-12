use async_trait::async_trait;
use sea_orm::{
    ActiveValue, FromJsonQueryResult, IntoSimpleExpr, QuerySelect, entity::prelude::*,
    sea_query::OnConflict,
};
use serde::{Deserialize, Serialize};

use super::{bangumi, query::InsertManyReturningExt, subscription_episode};
use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    extract::{
        mikan::{MikanEpisodeHash, MikanEpisodeMeta, build_mikan_episode_homepage_url},
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
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
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

impl ActiveModel {
    #[tracing::instrument(err, skip(ctx), fields(bangumi_id = ?bangumi.id, mikan_episode_id = ?episode.mikan_episode_id))]
    pub fn from_mikan_bangumi_and_episode_meta(
        ctx: &dyn AppContextTrait,
        bangumi: &bangumi::Model,
        episode: MikanEpisodeMeta,
    ) -> RecorderResult<Self> {
        let mikan_base_url = ctx.mikan().base_url().clone();
        let rawname_meta = parse_episode_meta_from_raw_name(&episode.episode_title)?;
        let homepage = build_mikan_episode_homepage_url(mikan_base_url, &episode.mikan_episode_id);

        Ok(Self {
            mikan_episode_id: ActiveValue::Set(Some(episode.mikan_episode_id)),
            raw_name: ActiveValue::Set(episode.episode_title.clone()),
            display_name: ActiveValue::Set(episode.episode_title.clone()),
            bangumi_id: ActiveValue::Set(bangumi.id),
            subscriber_id: ActiveValue::Set(bangumi.subscriber_id),
            resolution: ActiveValue::Set(rawname_meta.resolution),
            season: ActiveValue::Set(if rawname_meta.season > 0 {
                rawname_meta.season
            } else {
                bangumi.season
            }),
            season_raw: ActiveValue::Set(
                rawname_meta
                    .season_raw
                    .or_else(|| bangumi.season_raw.clone()),
            ),
            fansub: ActiveValue::Set(rawname_meta.fansub.or_else(|| bangumi.fansub.clone())),
            poster_link: ActiveValue::Set(bangumi.poster_link.clone()),
            episode_index: ActiveValue::Set(rawname_meta.episode_index),
            homepage: ActiveValue::Set(Some(homepage.to_string())),
            subtitle: ActiveValue::Set(rawname_meta.subtitle),
            source: ActiveValue::Set(rawname_meta.source),
            extra: ActiveValue::Set(EpisodeExtra {
                name_zh: rawname_meta.name_zh,
                name_en: rawname_meta.name_en,
                name_jp: rawname_meta.name_jp,
                s_name_en: rawname_meta.name_en_no_season,
                s_name_jp: rawname_meta.name_jp_no_season,
                s_name_zh: rawname_meta.name_zh_no_season,
            }),
            ..Default::default()
        })
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn get_existed_mikan_episode_list(
        ctx: &dyn AppContextTrait,
        ids: impl Iterator<Item = MikanEpisodeHash>,
        subscriber_id: i32,
        _subscription_id: i32,
    ) -> RecorderResult<impl Iterator<Item = (i32, MikanEpisodeHash, i32)>> {
        let db = ctx.db();

        Ok(Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::MikanEpisodeId)
            .column(Column::BangumiId)
            .filter(
                Expr::tuple([
                    Column::MikanEpisodeId.into_simple_expr(),
                    Column::SubscriberId.into_simple_expr(),
                ])
                .in_tuples(
                    ids.into_iter()
                        .map(|id| (id.mikan_episode_id, subscriber_id)),
                ),
            )
            .into_tuple::<(i32, String, i32)>()
            .all(db)
            .await?
            .into_iter()
            .map(|(episode_id, mikan_episode_id, bangumi_id)| {
                (
                    episode_id,
                    MikanEpisodeHash { mikan_episode_id },
                    bangumi_id,
                )
            }))
    }

    pub async fn add_mikan_episodes_for_subscription(
        ctx: &dyn AppContextTrait,
        creations: impl Iterator<Item = (&bangumi::Model, MikanEpisodeMeta)>,
        subscriber_id: i32,
        subscription_id: i32,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        let new_episode_active_modes: Vec<ActiveModel> = creations
            .map(|(bangumi, episode_meta)| {
                ActiveModel::from_mikan_bangumi_and_episode_meta(ctx, bangumi, episode_meta)
            })
            .collect::<Result<_, _>>()?;

        let new_episode_ids = Entity::insert_many(new_episode_active_modes)
            .on_conflict(
                OnConflict::columns([Column::MikanEpisodeId, Column::SubscriberId])
                    .update_columns([Column::RawName, Column::PosterLink, Column::Homepage])
                    .to_owned(),
            )
            .exec_with_returning_columns(db, [Column::Id])
            .await?
            .into_iter()
            .flat_map(|r| r.try_get_many_by_index::<i32>());

        subscription_episode::Model::add_episodes_for_subscription(
            ctx,
            new_episode_ids,
            subscriber_id,
            subscription_id,
        )
        .await?;

        Ok(())
    }
}
