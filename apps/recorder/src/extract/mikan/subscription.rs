use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use async_graphql::{InputObject, SimpleObject};
use fetch::fetch_bytes;
use futures::try_join;
use itertools::Itertools;
use maplit::hashmap;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QuerySelect,
    RelationTrait,
};
use serde::{Deserialize, Serialize};
use snafu::OptionExt;
use url::Url;

use super::scrape_mikan_bangumi_meta_list_from_season_flow_url;
use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::mikan::{
        MikanBangumiHash, MikanBangumiMeta, MikanEpisodeHash, MikanEpisodeMeta, MikanRssItem,
        MikanSeasonFlowUrlMeta, MikanSeasonStr, MikanSubscriberSubscriptionRssUrlMeta,
        build_mikan_bangumi_subscription_rss_url, build_mikan_season_flow_url,
        build_mikan_subscriber_subscription_rss_url,
        scrape_mikan_episode_meta_from_episode_homepage_url,
    },
    models::{
        bangumi, episodes, subscription_bangumi, subscription_episode,
        subscriptions::{self, SubscriptionTrait},
    },
};

#[tracing::instrument(err, skip(ctx, rss_item_list))]
async fn sync_mikan_feeds_from_rss_item_list(
    ctx: &dyn AppContextTrait,
    rss_item_list: Vec<MikanRssItem>,
    subscriber_id: i32,
    subscription_id: i32,
) -> RecorderResult<()> {
    let (new_episode_meta_list, existed_episode_hash2id_map) = {
        let existed_episode_hash2id_map = episodes::Model::get_existed_mikan_episode_list(
            ctx,
            rss_item_list.iter().map(|s| MikanEpisodeHash {
                mikan_episode_id: s.mikan_episode_id.clone(),
            }),
            subscriber_id,
            subscription_id,
        )
        .await?
        .map(|(episode_id, hash, bangumi_id)| (hash.mikan_episode_id, (episode_id, bangumi_id)))
        .collect::<HashMap<_, _>>();

        let mut new_episode_meta_list: Vec<MikanEpisodeMeta> = vec![];

        let mikan_client = ctx.mikan();
        for to_insert_rss_item in rss_item_list.into_iter().filter(|rss_item| {
            !existed_episode_hash2id_map.contains_key(&rss_item.mikan_episode_id)
        }) {
            let episode_meta = scrape_mikan_episode_meta_from_episode_homepage_url(
                mikan_client,
                to_insert_rss_item.homepage,
            )
            .await?;
            new_episode_meta_list.push(episode_meta);
        }

        (new_episode_meta_list, existed_episode_hash2id_map)
    };

    // subscribe existed but not subscribed episode and bangumi
    let (existed_episode_id_list, existed_episode_bangumi_id_set): (Vec<i32>, HashSet<i32>) =
        existed_episode_hash2id_map.into_values().unzip();

    try_join!(
        subscription_episode::Model::add_episodes_for_subscription(
            ctx,
            existed_episode_id_list.into_iter(),
            subscriber_id,
            subscription_id,
        ),
        subscription_bangumi::Model::add_bangumis_for_subscription(
            ctx,
            existed_episode_bangumi_id_set.into_iter(),
            subscriber_id,
            subscription_id,
        ),
    )?;

    let new_episode_meta_list_group_by_bangumi_hash: HashMap<
        MikanBangumiHash,
        Vec<MikanEpisodeMeta>,
    > = {
        let mut m = hashmap! {};
        for episode_meta in new_episode_meta_list {
            let bangumi_hash = episode_meta.bangumi_hash();

            m.entry(bangumi_hash)
                .or_insert_with(Vec::new)
                .push(episode_meta);
        }
        m
    };

    for (group_bangumi_hash, group_episode_meta_list) in new_episode_meta_list_group_by_bangumi_hash
    {
        let first_episode_meta = group_episode_meta_list.first().unwrap();
        let group_bangumi_model = bangumi::Model::get_or_insert_from_mikan(
            ctx,
            group_bangumi_hash,
            subscriber_id,
            subscription_id,
            async || {
                let bangumi_meta: MikanBangumiMeta = first_episode_meta.clone().into();
                let bangumi_am = bangumi::ActiveModel::from_mikan_bangumi_meta(
                    ctx,
                    bangumi_meta,
                    subscriber_id,
                    subscription_id,
                )
                .await?;
                Ok(bangumi_am)
            },
        )
        .await?;
        let group_episode_creation_list = group_episode_meta_list
            .into_iter()
            .map(|episode_meta| (&group_bangumi_model, episode_meta));

        episodes::Model::add_mikan_episodes_for_subscription(
            ctx,
            group_episode_creation_list.into_iter(),
            subscriber_id,
            subscription_id,
        )
        .await?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MikanSubscriberSubscription {
    pub id: i32,
    pub mikan_subscription_token: String,
    pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriptionTrait for MikanSubscriberSubscription {
    fn get_subscriber_id(&self) -> i32 {
        self.subscriber_id
    }

    fn get_subscription_id(&self) -> i32 {
        self.id
    }

    async fn sync_feeds(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let rss_item_list = self.get_rss_item_list(ctx.as_ref()).await?;

        sync_mikan_feeds_from_rss_item_list(
            ctx.as_ref(),
            rss_item_list,
            self.get_subscriber_id(),
            self.get_subscription_id(),
        )
        .await?;

        Ok(())
    }

    async fn sync_sources(&self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        Ok(())
    }

    fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
        let source_url = Url::parse(&model.source_url)?;

        let meta = MikanSubscriberSubscriptionRssUrlMeta::from_rss_url(&source_url)
            .with_whatever_context::<_, String, RecorderError>(|| {
                format!(
                    "MikanSubscriberSubscription should extract mikan_subscription_token from \
                     source_url = {}, subscription_id = {}",
                    source_url, model.id
                )
            })?;

        Ok(Self {
            id: model.id,
            mikan_subscription_token: meta.mikan_subscription_token,
            subscriber_id: model.subscriber_id,
        })
    }
}

impl MikanSubscriberSubscription {
    #[tracing::instrument(err, skip(ctx))]
    async fn get_rss_item_list(
        &self,
        ctx: &dyn AppContextTrait,
    ) -> RecorderResult<Vec<MikanRssItem>> {
        let mikan_base_url = ctx.mikan().base_url().clone();
        let rss_url = build_mikan_subscriber_subscription_rss_url(
            mikan_base_url.clone(),
            &self.mikan_subscription_token,
        );
        let bytes = fetch_bytes(ctx.mikan(), rss_url).await?;

        let channel = rss::Channel::read_from(&bytes[..])?;

        let mut result = vec![];
        for (idx, item) in channel.items.into_iter().enumerate() {
            let item = MikanRssItem::try_from(item).inspect_err(
                |error| tracing::warn!(error = %error, "failed to extract rss item idx = {}", idx),
            )?;
            result.push(item);
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, InputObject, SimpleObject)]
pub struct MikanSeasonSubscription {
    pub id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriptionTrait for MikanSeasonSubscription {
    fn get_subscriber_id(&self) -> i32 {
        self.subscriber_id
    }

    fn get_subscription_id(&self) -> i32 {
        self.id
    }

    async fn sync_feeds(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let rss_item_list = self.get_rss_item_list(ctx.as_ref()).await?;

        sync_mikan_feeds_from_rss_item_list(
            ctx.as_ref(),
            rss_item_list,
            self.get_subscriber_id(),
            self.get_subscription_id(),
        )
        .await?;

        Ok(())
    }

    async fn sync_sources(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let bangumi_meta_list = self.get_bangumi_meta_list(ctx.clone()).await?;

        let mikan_base_url = ctx.mikan().base_url();

        let rss_link_list = bangumi_meta_list
            .into_iter()
            .map(|bangumi_meta| {
                build_mikan_bangumi_subscription_rss_url(
                    mikan_base_url.clone(),
                    &bangumi_meta.mikan_bangumi_id,
                    Some(&bangumi_meta.mikan_fansub_id),
                )
                .to_string()
            })
            .collect_vec();

        subscriptions::Entity::update_many()
            .set(subscriptions::ActiveModel {
                source_urls: Set(Some(rss_link_list)),
                ..Default::default()
            })
            .filter(subscription_bangumi::Column::SubscriptionId.eq(self.id))
            .exec(ctx.db())
            .await?;

        Ok(())
    }

    fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
        let source_url = Url::parse(&model.source_url)?;

        let source_url_meta = MikanSeasonFlowUrlMeta::from_url(&source_url)
            .with_whatever_context::<_, String, RecorderError>(|| {
                format!(
                    "MikanSeasonSubscription should extract season_str and year from source_url, \
                     source_url = {}, subscription_id = {}",
                    source_url, model.id
                )
            })?;

        let credential_id = model
            .credential_id
            .with_whatever_context::<_, String, RecorderError>(|| {
                format!(
                    "MikanSeasonSubscription credential_id is required, subscription_id = {}",
                    model.id
                )
            })?;

        Ok(Self {
            id: model.id,
            year: source_url_meta.year,
            season_str: source_url_meta.season_str,
            credential_id,
            subscriber_id: model.subscriber_id,
        })
    }
}

impl MikanSeasonSubscription {
    #[tracing::instrument(err, skip(ctx))]
    async fn get_bangumi_meta_list(
        &self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> RecorderResult<Vec<MikanBangumiMeta>> {
        let credential_id = self.credential_id;
        let year = self.year;
        let season_str = self.season_str;

        let mikan_base_url = ctx.mikan().base_url().clone();
        let mikan_season_flow_url = build_mikan_season_flow_url(mikan_base_url, year, season_str);

        scrape_mikan_bangumi_meta_list_from_season_flow_url(
            ctx,
            mikan_season_flow_url,
            credential_id,
        )
        .await
    }

    #[tracing::instrument(err, skip(ctx))]
    async fn get_rss_item_list(
        &self,
        ctx: &dyn AppContextTrait,
    ) -> RecorderResult<Vec<MikanRssItem>> {
        let db = ctx.db();

        let subscribed_bangumi_list = bangumi::Entity::find()
            .filter(Condition::all().add(subscription_bangumi::Column::SubscriptionId.eq(self.id)))
            .join_rev(
                JoinType::InnerJoin,
                subscription_bangumi::Relation::Bangumi.def(),
            )
            .all(db)
            .await?;

        let mut rss_item_list = vec![];
        for subscribed_bangumi in subscribed_bangumi_list {
            let rss_url = subscribed_bangumi
                .rss_link
                .with_whatever_context::<_, String, RecorderError>(|| {
                    format!(
                        "MikanSeasonSubscription rss_link is required, subscription_id = {}",
                        self.id
                    )
                })?;
            let bytes = fetch_bytes(ctx.mikan(), rss_url).await?;

            let channel = rss::Channel::read_from(&bytes[..])?;

            for (idx, item) in channel.items.into_iter().enumerate() {
                let item = MikanRssItem::try_from(item).inspect_err(
                |error| tracing::warn!(error = %error, "failed to extract rss item idx = {}", idx),
            )?;
                rss_item_list.push(item);
            }
        }
        Ok(rss_item_list)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, InputObject, SimpleObject)]
pub struct MikanBangumiSubscription {
    pub id: i32,
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: String,
    pub subscriber_id: i32,
}

#[async_trait::async_trait]
impl SubscriptionTrait for MikanBangumiSubscription {
    fn get_subscriber_id(&self) -> i32 {
        self.subscriber_id
    }

    fn get_subscription_id(&self) -> i32 {
        self.id
    }

    async fn sync_feeds(&self, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        let rss_item_list = self.get_rss_item_list(ctx.as_ref()).await?;

        sync_mikan_feeds_from_rss_item_list(
            ctx.as_ref(),
            rss_item_list,
            <Self as SubscriptionTrait>::get_subscriber_id(self),
            <Self as SubscriptionTrait>::get_subscription_id(self),
        )
        .await?;

        Ok(())
    }

    async fn sync_sources(&self, _ctx: Arc<dyn AppContextTrait>) -> RecorderResult<()> {
        Ok(())
    }

    fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
        let source_url = Url::parse(&model.source_url)?;

        let meta = MikanBangumiHash::from_rss_url(&source_url)
            .with_whatever_context::<_, String, RecorderError>(|| {
                format!(
                    "MikanBangumiSubscription need to extract bangumi id and fansub id from \
                     source_url = {}, subscription_id = {}",
                    source_url, model.id
                )
            })?;

        Ok(Self {
            id: model.id,
            mikan_bangumi_id: meta.mikan_bangumi_id,
            mikan_fansub_id: meta.mikan_fansub_id,
            subscriber_id: model.subscriber_id,
        })
    }
}

impl MikanBangumiSubscription {
    #[tracing::instrument(err, skip(ctx))]
    async fn get_rss_item_list(
        &self,
        ctx: &dyn AppContextTrait,
    ) -> RecorderResult<Vec<MikanRssItem>> {
        let mikan_base_url = ctx.mikan().base_url().clone();
        let rss_url = build_mikan_bangumi_subscription_rss_url(
            mikan_base_url.clone(),
            &self.mikan_bangumi_id,
            Some(&self.mikan_fansub_id),
        );
        let bytes = fetch_bytes(ctx.mikan(), rss_url).await?;

        let channel = rss::Channel::read_from(&bytes[..])?;

        let mut result = vec![];
        for (idx, item) in channel.items.into_iter().enumerate() {
            let item = MikanRssItem::try_from(item).inspect_err(
                |error| tracing::warn!(error = %error, "failed to extract rss item idx = {}", idx),
            )?;
            result.push(item);
        }
        Ok(result)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use downloader::bittorrent::BITTORRENT_MIME_TYPE;
//     use rstest::rstest;
//     use url::Url;

//     use crate::{
//         errors::RecorderResult,
//         extract::mikan::{
//             MikanBangumiIndexRssChannel, MikanBangumiRssChannel,
// MikanRssChannel,             build_mikan_bangumi_subscription_rss_url,
// extract_mikan_rss_channel_from_rss_link,         },
//         test_utils::mikan::build_testing_mikan_client,
//     };

// #[rstest]
// #[tokio::test]
// async fn test_parse_mikan_rss_channel_from_rss_link() ->
// RecorderResult<()> {     let mut mikan_server =
// mockito::Server::new_async().await;

//     let mikan_base_url = Url::parse(&mikan_server.url())?;

//     let mikan_client =
// build_testing_mikan_client(mikan_base_url.clone()).await?;

//     {
//         let bangumi_rss_url = build_mikan_bangumi_subscription_rss_url(
//             mikan_base_url.clone(),
//             "3141",
//             Some("370"),
//         );

//         let bangumi_rss_mock = mikan_server
//             .mock("GET", bangumi_rss_url.path())
//
// .with_body_from_file("tests/resources/mikan/Bangumi-3141-370.rss")
//             .match_query(mockito::Matcher::Any)
//             .create_async()
//             .await;

//         let channel =
// scrape_mikan_rss_channel_from_rss_link(&mikan_client, bangumi_rss_url)
//             .await
//             .expect("should get mikan channel from rss url");

//         assert_matches!(
//             &channel,
//             MikanRssChannel::Bangumi(MikanBangumiRssChannel { .. })
//         );

//         assert_matches!(&channel.name(), Some("葬送的芙莉莲"));

//         let items = channel.items();
//         let first_sub_item = items
//             .first()
//             .expect("mikan subscriptions should have at least one subs");

//         assert_eq!(first_sub_item.mime, BITTORRENT_MIME_TYPE);

//         assert!(
//             &first_sub_item
//                 .homepage
//                 .as_str()
//                 .starts_with("https://mikanani.me/Home/Episode")
//         );

//         let name = first_sub_item.title.as_str();
//         assert!(name.contains("葬送的芙莉莲"));

//         bangumi_rss_mock.expect(1);
//     }
//     {
//         let bangumi_rss_url =
// mikan_base_url.join("/RSS/Bangumi?bangumiId=3416")?;

//         let bangumi_rss_mock = mikan_server
//             .mock("GET", bangumi_rss_url.path())
//             .match_query(mockito::Matcher::Any)
//
// .with_body_from_file("tests/resources/mikan/Bangumi-3416.rss")
//             .create_async()
//             .await;

//         let channel =
// scrape_mikan_rss_channel_from_rss_link(&mikan_client, bangumi_rss_url)
//             .await
//             .expect("should get mikan channel from rss url");

//         assert_matches!(
//             &channel,
//             MikanRssChannel::BangumiIndex(MikanBangumiIndexRssChannel {
// .. })         );

//         assert_matches!(&channel.name(), Some("叹气的亡灵想隐退"));

//         bangumi_rss_mock.expect(1);
//     }
//     Ok(())
// }
// }
