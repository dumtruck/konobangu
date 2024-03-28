use std::collections::HashMap;

use itertools::Itertools;
use loco_rs::app::AppContext;
use sea_orm::{
    entity::prelude::*,
    sea_query::{InsertStatement, OnConflict},
    ActiveValue,
};
use serde::{Deserialize, Serialize};
use tracing::{event, instrument, Level};

pub use super::entities::subscriptions::{self, *};
use crate::{
    models::{bangumi, db_utils::insert_many_with_returning_all, downloads, episodes},
    parsers::{
        mikan::{
            parse_episode_meta_from_mikan_homepage, parse_mikan_rss_items_from_rss_link,
            MikanClient, MikanEpisodeMeta,
        },
        raw::{parse_episode_meta_from_raw_name, RawEpisodeMeta},
    },
    path::extract_extname_from_url,
    storage::{AppContextDalExt, DalContentType},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionCreateFromRssDto {
    pub rss_link: String,
    pub display_name: String,
    pub aggregate: bool,
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
            aggregate: ActiveValue::Set(create_dto.aggregate),
            subscriber_id: ActiveValue::Set(subscriber_id),
            category: ActiveValue::Set(category),
            source_url: ActiveValue::Set(create_dto.rss_link),
            ..Default::default()
        }
    }
}

impl Model {
    pub async fn add_subscription(
        db: &DatabaseConnection,
        create_dto: SubscriptionCreateDto,
        subscriber_id: i32,
    ) -> eyre::Result<Self> {
        let subscription = ActiveModel::from_create_dto(create_dto, subscriber_id);

        Ok(subscription.insert(db).await?)
    }

    pub async fn toggle_iters(
        db: &DatabaseConnection,
        ids: impl Iterator<Item = i32>,
        enabled: bool,
    ) -> eyre::Result<()> {
        Entity::update_many()
            .col_expr(Column::Enabled, Expr::value(enabled))
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_iters(
        db: &DatabaseConnection,
        ids: impl Iterator<Item = i32>,
    ) -> eyre::Result<()> {
        Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    #[instrument(
        fields(subscriber_id = "self.subscriber_id", subscription_id = "self.id"),
        skip(self, db, ctx)
    )]
    pub async fn pull_item(&self, db: &DatabaseConnection, ctx: &AppContext) -> eyre::Result<()> {
        let subscription = self;
        let subscription_id = subscription.id;
        match &subscription.category {
            SubscriptionCategory::Mikan => {
                let subscriber_id = subscription.subscriber_id;
                let mikan_client = MikanClient::new(subscriber_id).await?;
                let mikan_rss_items =
                    parse_mikan_rss_items_from_rss_link(&mikan_client, &subscription.source_url)
                        .await?;
                let all_items = mikan_rss_items.collect::<Vec<_>>();

                if all_items.is_empty() {
                    return Ok(());
                }

                let new_downloads = all_items
                    .into_iter()
                    .map(|rss_item| {
                        downloads::ActiveModel::from_mikan_rss_item(rss_item, subscription.id)
                    })
                    .collect_vec();

                // insert and filter out duplicated items
                let new_downloads: Vec<downloads::Model> = insert_many_with_returning_all(
                    db,
                    new_downloads,
                    |stat: &mut InsertStatement| {
                        stat.on_conflict(
                            OnConflict::column(downloads::Column::Url)
                                .do_nothing()
                                .to_owned(),
                        );
                    },
                )
                .await?;

                pub struct MikanEpMetaBundle {
                    pub download: downloads::Model,
                    pub mikan: MikanEpisodeMeta,
                    pub raw: RawEpisodeMeta,
                    pub poster: Option<String>,
                }

                let mut ep_metas: HashMap<bangumi::BangumiUniqueKey, Vec<MikanEpMetaBundle>> =
                    HashMap::new();
                let dal = ctx.get_dal_unwrap().await;
                {
                    for dl in new_downloads {
                        let mut mikan_meta = if let Some(homepage) = dl.homepage.as_deref() {
                            match parse_episode_meta_from_mikan_homepage(&mikan_client, homepage)
                                .await
                            {
                                Ok(mikan_meta) => mikan_meta,
                                Err(e) => {
                                    let error: &dyn std::error::Error = e.as_ref();
                                    event!(
                                        Level::ERROR,
                                        desc = "failed to parse episode meta from mikan homepage",
                                        homepage = homepage,
                                        error = error
                                    );
                                    continue;
                                }
                            }
                        } else {
                            continue;
                        };
                        let mikan_poster_link = if let Some(poster) = mikan_meta.poster.take() {
                            if let Some(extname) = extract_extname_from_url(&poster.origin_url) {
                                let result = dal
                                    .store_blob(
                                        DalContentType::Poster,
                                        &extname,
                                        poster.data,
                                        &subscriber_id.to_string(),
                                    )
                                    .await;
                                match result {
                                    Ok(stored_url) => Some(stored_url.to_string()),
                                    Err(e) => {
                                        let error: &dyn std::error::Error = e.as_ref();
                                        event!(
                                            Level::ERROR,
                                            desc = "failed to store mikan meta poster",
                                            origin_url = poster.origin_url.as_str(),
                                            error = error
                                        );
                                        None
                                    }
                                }
                            } else {
                                event!(
                                    Level::ERROR,
                                    desc = "failed to extract mikan meta poster extname",
                                    origin_url = poster.origin_url.as_str(),
                                );
                                None
                            }
                        } else {
                            None
                        };
                        let raw_meta = match parse_episode_meta_from_raw_name(&dl.origin_title) {
                            Ok(raw_meta) => raw_meta,
                            Err(e) => {
                                let error: &dyn std::error::Error = e.as_ref();
                                event!(
                                    Level::ERROR,
                                    desc = "failed to parse episode meta from origin name",
                                    origin_name = &dl.origin_title,
                                    error = error
                                );
                                continue;
                            }
                        };
                        let key = bangumi::BangumiUniqueKey {
                            official_title: mikan_meta.official_title.clone(),
                            season: raw_meta.season,
                            fansub: raw_meta.fansub.clone(),
                        };
                        let meta = MikanEpMetaBundle {
                            download: dl,
                            mikan: mikan_meta,
                            raw: raw_meta,
                            poster: mikan_poster_link,
                        };
                        ep_metas.entry(key).or_default().push(meta);
                    }
                }

                for (_, eps) in ep_metas {
                    let meta = eps.first().unwrap_or_else(|| {
                        unreachable!(
                            "subscriptions pull items bangumi must have at least one episode meta"
                        )
                    });
                    let last_ep = eps.iter().fold(0, |acc, ep| acc.max(ep.raw.episode_index));
                    let official_title = &meta.mikan.official_title;
                    let bgm = bangumi::ActiveModel {
                        subscription_id: ActiveValue::Set(subscription_id),
                        display_name: ActiveValue::Set(official_title.clone()),
                        official_title: ActiveValue::Set(official_title.clone()),
                        fansub: ActiveValue::Set(meta.raw.fansub.clone()),
                        season: ActiveValue::Set(meta.raw.season),
                        poster_link: ActiveValue::Set(meta.poster.clone()),
                        last_ep: ActiveValue::Set(last_ep),
                        ..Default::default()
                    };

                    let bgm = bangumi::Entity::insert(bgm)
                        .on_conflict(
                            OnConflict::columns([
                                bangumi::Column::OfficialTitle,
                                bangumi::Column::Season,
                                bangumi::Column::Fansub,
                            ])
                            .update_columns([bangumi::Column::LastEp])
                            .to_owned(),
                        )
                        .exec_with_returning(db)
                        .await?;

                    let eps = eps.into_iter().map(|ep| {
                        episodes::ActiveModel::from_mikan_meta(
                            bgm.id,
                            ep.download,
                            ep.raw,
                            ep.mikan,
                            ep.poster,
                        )
                    });
                    episodes::Entity::insert_many(eps).exec(db).await?;
                }

                Ok(())
            }
            _ => {
                todo!("other subscription categories")
            }
        }
    }
}
