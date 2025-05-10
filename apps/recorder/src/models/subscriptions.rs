use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use itertools::Itertools;
use sea_orm::{ActiveValue, entity::prelude::*};
use serde::{Deserialize, Serialize};

use super::{bangumi, episodes, query::filter_values_in};
use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::{
        mikan::{
            MikanBangumiPosterMeta, MikanBangumiSubscription, MikanSeasonSubscription,
            MikanSubscriberSubscription, build_mikan_bangumi_homepage_url,
            build_mikan_bangumi_subscription_rss_url,
            scrape_mikan_bangumi_meta_from_bangumi_homepage_url,
            scrape_mikan_episode_meta_from_episode_homepage_url,
            scrape_mikan_poster_meta_from_image_url,
        },
        rawname::extract_season_from_title_body,
    },
    models::episodes::MikanEpsiodeCreation,
};

#[derive(
    Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, DeriveDisplay,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "subscription_category"
)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionCategory {
    #[sea_orm(string_value = "mikan_subscriber")]
    MikanSubscriber,
    #[sea_orm(string_value = "mikan_season")]
    MikanSeason,
    #[sea_orm(string_value = "mikan_bangumi")]
    MikanBangumi,
    #[sea_orm(string_value = "manual")]
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "category")]
pub enum SubscriptionPayload {
    #[serde(rename = "mikan_subscriber")]
    MikanSubscriber(MikanSubscriberSubscription),
    #[serde(rename = "mikan_season")]
    MikanSeason(MikanSeasonSubscription),
    #[serde(rename = "mikan_bangumi")]
    MikanBangumi(MikanBangumiSubscription),
    #[serde(rename = "manual")]
    Manual,
}

impl SubscriptionPayload {
    pub fn category(&self) -> SubscriptionCategory {
        match self {
            Self::MikanSubscriber(_) => SubscriptionCategory::MikanSubscriber,
            Self::MikanSeason(_) => SubscriptionCategory::MikanSeason,
            Self::MikanBangumi(_) => SubscriptionCategory::MikanBangumi,
            Self::Manual => SubscriptionCategory::Manual,
        }
    }

    pub fn try_from_model(model: &Model) -> RecorderResult<Self> {
        Ok(match model.category {
            SubscriptionCategory::MikanSubscriber => {
                Self::MikanSubscriber(MikanSubscriberSubscription::try_from_model(model)?)
            }
            SubscriptionCategory::MikanSeason => {
                Self::MikanSeason(MikanSeasonSubscription::try_from_model(model)?)
            }
            SubscriptionCategory::MikanBangumi => {
                Self::MikanBangumi(MikanBangumiSubscription::try_from_model(model)?)
            }
            SubscriptionCategory::Manual => Self::Manual,
        })
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    pub subscriber_id: i32,
    pub category: SubscriptionCategory,
    pub source_url: String,
    pub enabled: bool,
    pub credential_id: Option<i32>,
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
    #[sea_orm(has_many = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episodes,
    #[sea_orm(has_many = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
    #[sea_orm(has_many = "super::subscription_bangumi::Entity")]
    SubscriptionBangumi,
    #[sea_orm(
        belongs_to = "super::credential_3rd::Entity",
        from = "Column::CredentialId",
        to = "super::credential_3rd::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Credential3rd,
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::subscription_bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionBangumi.def()
    }
}

impl Related<super::subscription_episode::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionEpisode.def()
    }
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_bangumi::Relation::Bangumi.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::subscription_bangumi::Relation::Subscription
                .def()
                .rev(),
        )
    }
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_episode::Relation::Episode.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::subscription_episode::Relation::Subscription
                .def()
                .rev(),
        )
    }
}

impl Related<super::credential_3rd::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Credential3rd.def()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(entity = "super::episodes::Entity")]
    Episode,
    #[sea_orm(entity = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
    #[sea_orm(entity = "super::subscription_bangumi::Entity")]
    SubscriptionBangumi,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {}

impl Model {
    pub async fn find_by_id(ctx: &dyn AppContextTrait, id: i32) -> RecorderResult<Option<Self>> {
        let db = ctx.db();
        Ok(Entity::find_by_id(id).one(db).await?)
    }

    pub async fn toggle_with_ids(
        ctx: &dyn AppContextTrait,
        ids: impl Iterator<Item = i32>,
        enabled: bool,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        Entity::update_many()
            .col_expr(Column::Enabled, Expr::value(enabled))
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_with_ids(
        ctx: &dyn AppContextTrait,
        ids: impl Iterator<Item = i32>,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn pull_subscription(&self, ctx: &dyn AppContextTrait) -> RecorderResult<()> {
        match payload {
            SubscriptionPayload::MikanSubscriber(payload) => {
                let mikan_client = ctx.mikan();
                let channel =
                    extract_mikan_rss_channel_from_rss_link(mikan_client, &self.source_url).await?;

                let items = channel.into_items();

                let db = ctx.db();
                let items = items.into_iter().collect_vec();

                let mut stmt = filter_values_in(
                    episodes::Entity,
                    episodes::Column::MikanEpisodeId,
                    items
                        .iter()
                        .map(|s| Value::from(s.mikan_episode_id.clone())),
                );
                stmt.and_where(Expr::col(episodes::Column::SubscriberId).eq(self.subscriber_id));

                let builder = &db.get_database_backend();

                let old_rss_item_mikan_episode_ids_set = db
                    .query_all(builder.build(&stmt))
                    .await?
                    .into_iter()
                    .flat_map(|qs| qs.try_get_by_index(0))
                    .collect::<HashSet<String>>();

                let new_rss_items = items
                    .into_iter()
                    .filter(|item| {
                        !old_rss_item_mikan_episode_ids_set.contains(&item.mikan_episode_id)
                    })
                    .collect_vec();

                let mut new_metas = vec![];
                for new_rss_item in new_rss_items.iter() {
                    new_metas.push(
                        scrape_mikan_episode_meta_from_episode_homepage_url(
                            mikan_client,
                            new_rss_item.homepage.clone(),
                        )
                        .await?,
                    );
                }

                let new_mikan_bangumi_groups = new_metas
                    .into_iter()
                    .into_group_map_by(|s| (s.mikan_bangumi_id.clone(), s.mikan_fansub_id.clone()));

                for ((mikan_bangumi_id, mikan_fansub_id), new_ep_metas) in new_mikan_bangumi_groups
                {
                    let mikan_base_url = ctx.mikan().base_url();
                    let bgm_homepage = build_mikan_bangumi_homepage_url(
                        mikan_base_url.clone(),
                        &mikan_bangumi_id,
                        Some(&mikan_fansub_id),
                    );
                    let bgm_rss_link = build_mikan_bangumi_subscription_rss_url(
                        mikan_base_url.clone(),
                        &mikan_bangumi_id,
                        Some(&mikan_fansub_id),
                    )?;
                    let bgm = Arc::new(
                        bangumi::Model::get_or_insert_from_mikan(
                            ctx,
                            self.subscriber_id,
                            self.id,
                            mikan_bangumi_id.to_string(),
                            mikan_fansub_id.to_string(),
                            async |am| -> RecorderResult<()> {
                                let bgm_meta = scrape_mikan_bangumi_meta_from_bangumi_homepage_url(
                                    mikan_client,
                                    bgm_homepage.clone(),
                                )
                                .await?;
                                let bgm_name = bgm_meta.bangumi_title;
                                let (_, bgm_season_raw, bgm_season) =
                                    extract_season_from_title_body(&bgm_name);
                                am.raw_name = ActiveValue::Set(bgm_name.clone());
                                am.display_name = ActiveValue::Set(bgm_name);
                                am.season = ActiveValue::Set(bgm_season);
                                am.season_raw = ActiveValue::Set(bgm_season_raw);
                                am.rss_link = ActiveValue::Set(Some(bgm_rss_link.to_string()));
                                am.homepage = ActiveValue::Set(Some(bgm_homepage.to_string()));
                                am.fansub = ActiveValue::Set(Some(bgm_meta.fansub));
                                if let Some(origin_poster_src) = bgm_meta.origin_poster_src
                                    && let MikanBangumiPosterMeta {
                                        poster_src: Some(poster_src),
                                        ..
                                    } = scrape_mikan_poster_meta_from_image_url(
                                        mikan_client,
                                        ctx.storage(),
                                        origin_poster_src,
                                        self.subscriber_id,
                                    )
                                    .await?
                                {
                                    am.poster_link = ActiveValue::Set(Some(poster_src))
                                }
                                Ok(())
                            },
                        )
                        .await?,
                    );
                    episodes::Model::add_episodes(
                        ctx,
                        self.subscriber_id,
                        self.id,
                        new_ep_metas.into_iter().map(|item| MikanEpsiodeCreation {
                            episode: item,
                            bangumi: bgm.clone(),
                        }),
                    )
                    .await?;
                }
                Ok(())
            }
            _ => todo!(),
        }
    }
}
