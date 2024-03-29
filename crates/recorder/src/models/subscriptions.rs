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
    #[sea_orm(string_value = "mikan")]
    Mikan,
    #[sea_orm(string_value = "tmdb")]
    Tmdb,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(column_type = "Timestamp")]
    pub created_at: DateTime,
    #[sea_orm(column_type = "Timestamp")]
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    pub subscriber_id: i32,
    pub category: SubscriptionCategory,
    pub source_url: String,
    pub aggregate: bool,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id"
    )]
    Subscriber,
    #[sea_orm(has_many = "super::bangumi::Entity")]
    Bangumi,
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

use crate::{
    models::{bangumi, episodes, resources, subscribers},
    parsers::{
        mikan::{
            parse_episode_meta_from_mikan_homepage, parse_mikan_rss_items_from_rss_link,
            MikanClient, MikanEpisodeMeta,
        },
        raw::{parse_episode_meta_from_raw_name, RawEpisodeMeta},
    },
    utils::db::insert_many_with_returning_all,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionCreateFromRssDto {
    pub rss_link: String,
    pub display_name: String,
    pub aggregate: bool,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "category", rename_all = "snake_case")]
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
        skip(self, ctx)
    )]
    pub async fn pull_one(
        &self,
        ctx: &AppContext,
        subscriber: &subscribers::Model,
    ) -> eyre::Result<()> {
        let db = &ctx.db;
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

                let new_resources = all_items
                    .into_iter()
                    .map(|rss_item| {
                        resources::ActiveModel::from_mikan_rss_item(rss_item, subscription.id)
                    })
                    .collect_vec();

                // insert and filter out duplicated items
                let new_resources: Vec<resources::Model> = insert_many_with_returning_all(
                    db,
                    new_resources,
                    |stat: &mut InsertStatement| {
                        stat.on_conflict(
                            OnConflict::column(resources::Column::Url)
                                .do_nothing()
                                .to_owned(),
                        );
                    },
                )
                .await?;

                pub struct MikanEpMetaBundle {
                    pub resource: resources::Model,
                    pub mikan: MikanEpisodeMeta,
                    pub raw: RawEpisodeMeta,
                    pub poster: Option<String>,
                }

                let mut ep_metas: HashMap<bangumi::BangumiUniqueKey, Vec<MikanEpMetaBundle>> =
                    HashMap::new();
                {
                    for r in new_resources {
                        let mut mikan_meta = if let Some(homepage) = r.homepage.as_deref() {
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
                        let mikan_poster_link =
                            if let Some(poster_url) = mikan_meta.poster_url.take() {
                                let poster_url_str = poster_url.to_string();
                                let poster_resource_result = resources::Model::from_poster_url(
                                    ctx,
                                    &subscriber.pid,
                                    subscription_id,
                                    mikan_meta.official_title.clone(),
                                    poster_url,
                                    |url| mikan_client.fetch_bytes(|f| f.get(url)),
                                )
                                .await;
                                match poster_resource_result {
                                    Ok(resource) => resource.save_path,
                                    Err(e) => {
                                        let error: &dyn std::error::Error = e.as_ref();
                                        event!(
                                            Level::ERROR,
                                            desc = "failed to fetch mikan meta poster",
                                            poster_url = poster_url_str,
                                            error = error
                                        );
                                        None
                                    }
                                }
                            } else {
                                None
                            };
                        let raw_meta = match parse_episode_meta_from_raw_name(&r.origin_title) {
                            Ok(raw_meta) => raw_meta,
                            Err(e) => {
                                let error: &dyn std::error::Error = e.as_ref();
                                event!(
                                    Level::ERROR,
                                    desc = "failed to parse episode meta from origin name",
                                    origin_name = &r.origin_title,
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
                            resource: r,
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
                            "subscriptions pull one bangumi must have at least one episode meta"
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
                            ep.resource,
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
