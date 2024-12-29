use std::{collections::HashSet, sync::Arc};

use itertools::Itertools;
use loco_rs::app::AppContext;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

pub use super::entities::subscriptions::{self, *};
use super::{bangumi, episodes, query::filter_values_in};
use crate::{
    app::AppContextExt,
    extract::{
        mikan::{
            build_mikan_bangumi_homepage, build_mikan_bangumi_rss_link,
            parse_mikan_bangumi_meta_from_mikan_homepage,
            parse_mikan_episode_meta_from_mikan_homepage, parse_mikan_rss_channel_from_rss_link,
            web_parser::{
                parse_mikan_bangumi_poster_from_origin_poster_src_with_cache,
                MikanBangumiPosterMeta,
            },
        },
        rawname::extract_season_from_title_body,
    },
    models::episodes::MikanEpsiodeCreation,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionCreateFromRssDto {
    pub rss_link: String,
    pub display_name: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "category")]
pub enum SubscriptionCreateDto {
    Mikan(SubscriptionCreateFromRssDto),
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_create_dto(create_dto: SubscriptionCreateDto, subscriber_id: i32) -> Self {
        match create_dto {
            SubscriptionCreateDto::Mikan(create_dto) => {
                Self::from_rss_create_dto(SubscriptionCategory::Mikan, create_dto, subscriber_id)
            }
        }
    }

    fn from_rss_create_dto(
        category: SubscriptionCategory,
        create_dto: SubscriptionCreateFromRssDto,
        subscriber_id: i32,
    ) -> Self {
        Self {
            display_name: ActiveValue::Set(create_dto.display_name),
            enabled: ActiveValue::Set(create_dto.enabled.unwrap_or(false)),
            subscriber_id: ActiveValue::Set(subscriber_id),
            category: ActiveValue::Set(category),
            source_url: ActiveValue::Set(create_dto.rss_link),
            ..Default::default()
        }
    }
}

impl Model {
    pub async fn add_subscription(
        ctx: &AppContext,
        create_dto: SubscriptionCreateDto,
        subscriber_id: i32,
    ) -> eyre::Result<Self> {
        let db = &ctx.db;
        let subscription = ActiveModel::from_create_dto(create_dto, subscriber_id);

        Ok(subscription.insert(db).await?)
    }

    pub async fn toggle_iters(
        ctx: &AppContext,
        ids: impl Iterator<Item = i32>,
        enabled: bool,
    ) -> eyre::Result<()> {
        let db = &ctx.db;
        Entity::update_many()
            .col_expr(Column::Enabled, Expr::value(enabled))
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_iters(
        ctx: &AppContext,
        ids: impl Iterator<Item = i32>,
    ) -> eyre::Result<()> {
        let db = &ctx.db;
        Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn pull_subscription(&self, ctx: &AppContext) -> eyre::Result<()> {
        match &self.category {
            SubscriptionCategory::Mikan => {
                let mikan_client = ctx.get_mikan_client();
                let channel =
                    parse_mikan_rss_channel_from_rss_link(Some(mikan_client), &self.source_url)
                        .await?;

                let items = channel.into_items();

                let db = &ctx.db;
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
                        parse_mikan_episode_meta_from_mikan_homepage(
                            Some(mikan_client),
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
                    let mikan_base_url = ctx.get_mikan_client().base_url();
                    let bgm_homepage = build_mikan_bangumi_homepage(
                        mikan_base_url,
                        &mikan_bangumi_id,
                        Some(&mikan_fansub_id),
                    )?;
                    let bgm_rss_link = build_mikan_bangumi_rss_link(
                        mikan_base_url,
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
                            async |am| -> eyre::Result<()> {
                                let bgm_meta = parse_mikan_bangumi_meta_from_mikan_homepage(
                                    Some(mikan_client),
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
                                am.fansub = ActiveValue::Set(bgm_meta.fansub);
                                if let Some(origin_poster_src) = bgm_meta.origin_poster_src {
                                    if let MikanBangumiPosterMeta {
                                    poster_src: Some(poster_src),
                                    ..
                                    } = parse_mikan_bangumi_poster_from_origin_poster_src_with_cache(
                                        ctx,
                                        origin_poster_src,
                                        self.subscriber_id,
                                    )
                                    .await?
                                    {
                                        am.poster_link = ActiveValue::Set(Some(poster_src))
                                    }
                                }
                            Ok(())
                            },
                        )
                        .await?,
                    );
                    episodes::Model::add_episodes(
                        ctx,
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
