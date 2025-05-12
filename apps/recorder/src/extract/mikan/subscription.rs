use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_graphql::{InputObject, SimpleObject};
use async_stream::try_stream;
use fetch::{fetch_bytes, fetch_html};
use futures::Stream;
use itertools::Itertools;
use maplit::hashmap;
use scraper::Html;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoSimpleExpr, QueryFilter,
    QuerySelect, prelude::Expr, sea_query::OnConflict,
};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};
use url::Url;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    extract::mikan::{
        MikanBangumiHash, MikanBangumiMeta, MikanBangumiRssUrlMeta, MikanEpisodeHash,
        MikanEpisodeMeta, MikanRssItem, MikanSeasonFlowUrlMeta, MikanSeasonStr,
        MikanSubscriberSubscriptionRssUrlMeta, build_mikan_bangumi_expand_subscribed_url,
        build_mikan_bangumi_subscription_rss_url, build_mikan_season_flow_url,
        build_mikan_subscriber_subscription_rss_url,
        extract_mikan_bangumi_index_meta_list_from_season_flow_fragment,
        extract_mikan_bangumi_meta_from_expand_subscribed_fragment,
        scrape_mikan_episode_meta_from_episode_homepage_url,
    },
    migrations::defs::Bangumi,
    models::{bangumi, episodes, subscription_bangumi, subscription_episode, subscriptions},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, InputObject, SimpleObject)]
pub struct MikanSubscriberSubscription {
    pub id: i32,
    pub mikan_subscription_token: String,
    pub subscriber_id: i32,
}

impl MikanSubscriberSubscription {
    #[tracing::instrument(skip(ctx))]
    pub async fn pull_subscription(
        &self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> RecorderResult<Vec<MikanBangumiMeta>> {
        let mikan_client = ctx.mikan();
        let db = ctx.db();

        let to_insert_episode_meta_list: Vec<MikanEpisodeMeta> = {
            let rss_item_list = self.pull_rss_items(ctx.clone()).await?;

            let existed_episode_token_list = episodes::Model::get_existed_mikan_episode_list(
                ctx.as_ref(),
                rss_item_list.iter().map(|s| MikanEpisodeHash {
                    mikan_episode_token: s.mikan_episode_id.clone(),
                }),
                self.subscriber_id,
                self.id,
            )
            .await?
            .into_iter()
            .map(|(id, hash)| (hash.mikan_episode_token, id))
            .collect::<HashMap<_, _>>();

            let mut to_insert_episode_meta_list = vec![];

            for to_insert_rss_item in rss_item_list.into_iter().filter(|rss_item| {
                !existed_episode_token_list.contains_key(&rss_item.mikan_episode_id)
            }) {
                let episode_meta = scrape_mikan_episode_meta_from_episode_homepage_url(
                    mikan_client,
                    to_insert_rss_item.homepage,
                )
                .await?;
                to_insert_episode_meta_list.push(episode_meta);
            }

            subscription_episode::Model::add_episodes_for_subscription(
                ctx.as_ref(),
                existed_episode_token_list.into_values(),
                self.subscriber_id,
                self.id,
            )
            .await?;

            to_insert_episode_meta_list
        };

        let new_episode_meta_bangumi_map = {
            let bangumi_hash_map = to_insert_episode_meta_list
                .iter()
                .map(|episode_meta| (episode_meta.bangumi_hash(), episode_meta))
                .collect::<HashMap<_, _>>();

            let existed_bangumi_set = bangumi::Model::get_existed_mikan_bangumi_list(
                ctx.as_ref(),
                bangumi_hash_map.keys().cloned(),
                self.subscriber_id,
                self.id,
            )
            .await?
            .map(|(_, bangumi_hash)| bangumi_hash)
            .collect::<HashSet<_>>();

            let mut to_insert_bangumi_list = vec![];

            for (bangumi_hash, episode_meta) in bangumi_hash_map.iter() {
                if !existed_bangumi_set.contains(&bangumi_hash) {
                    let bangumi_meta: MikanBangumiMeta = (*episode_meta).clone().into();

                    let bangumi_active_model = bangumi::ActiveModel::from_mikan_bangumi_meta(
                        ctx.as_ref(),
                        bangumi_meta,
                        self.subscriber_id,
                        self.id,
                    )
                    .await?;

                    to_insert_bangumi_list.push(bangumi_active_model);
                }
            }

            bangumi::Entity::insert_many(to_insert_bangumi_list)
                .on_conflict_do_nothing()
                .exec(db)
                .await?;

            let mut new_episode_meta_bangumi_map: HashMap<MikanBangumiHash, bangumi::Model> =
                hashmap! {};
        };

        todo!()
    }

    #[tracing::instrument(skip(ctx))]
    pub async fn pull_rss_items(
        &self,
        ctx: Arc<dyn AppContextTrait>,
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

    pub fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
        let source_url = Url::parse(&model.source_url)?;

        let meta = MikanSubscriberSubscriptionRssUrlMeta::from_url(&source_url)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, InputObject, SimpleObject)]
pub struct MikanSeasonSubscription {
    pub id: i32,
    pub year: i32,
    pub season_str: MikanSeasonStr,
    pub credential_id: i32,
    pub subscriber_id: i32,
}

impl MikanSeasonSubscription {
    #[tracing::instrument]
    pub fn pull_bangumi_meta_stream(
        &self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = RecorderResult<MikanBangumiMeta>> {
        let credential_id = self.credential_id;
        let year = self.year;
        let season_str = self.season_str.clone();

        try_stream! {
            let mikan_base_url = ctx.mikan().base_url().clone();

            let mikan_client = ctx.mikan()
            .fork_with_credential(ctx.clone(), credential_id)
            .await?;

            let mikan_season_flow_url = build_mikan_season_flow_url(mikan_base_url.clone(), year, season_str);

            let content = fetch_html(&mikan_client, mikan_season_flow_url.clone()).await?;

            let mut bangumi_indices_meta = {
                let html = Html::parse_document(&content);
                extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, &mikan_base_url)
            };

            if bangumi_indices_meta.is_empty() && !mikan_client.has_login().await? {
                mikan_client.login().await?;
                let content = fetch_html(&mikan_client, mikan_season_flow_url).await?;
                let html = Html::parse_document(&content);
                bangumi_indices_meta =
                    extract_mikan_bangumi_index_meta_list_from_season_flow_fragment(&html, &mikan_base_url);
            }


            mikan_client
                .sync_credential_cookies(ctx.clone(), credential_id)
                .await?;

            for bangumi_index in bangumi_indices_meta {
                let bangumi_title = bangumi_index.bangumi_title.clone();
                let bangumi_expand_subscribed_fragment_url = build_mikan_bangumi_expand_subscribed_url(
                    mikan_base_url.clone(),
                    &bangumi_index.mikan_bangumi_id,
                );
                let bangumi_expand_subscribed_fragment =
                    fetch_html(&mikan_client, bangumi_expand_subscribed_fragment_url).await?;

                let bangumi_meta = {
                    let html = Html::parse_document(&bangumi_expand_subscribed_fragment);

                    extract_mikan_bangumi_meta_from_expand_subscribed_fragment(
                        &html,
                        bangumi_index,
                        mikan_base_url.clone(),
                    )
                    .with_whatever_context::<_, String, RecorderError>(|| {
                        format!("failed to extract mikan bangumi fansub of title = {bangumi_title}")
                    })
                }?;

                yield bangumi_meta;
            }

            mikan_client
            .sync_credential_cookies(ctx, credential_id)
            .await?;
        }
    }

    pub fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, InputObject, SimpleObject)]
pub struct MikanBangumiSubscription {
    pub id: i32,
    pub mikan_bangumi_id: String,
    pub mikan_fansub_id: String,
    pub subscriber_id: i32,
}

impl MikanBangumiSubscription {
    #[tracing::instrument]
    pub fn pull_rss_items(
        &self,
        ctx: Arc<dyn AppContextTrait>,
    ) -> impl Stream<Item = RecorderResult<MikanRssItem>> {
        let mikan_bangumi_id = self.mikan_bangumi_id.clone();
        let mikan_fansub_id = self.mikan_fansub_id.clone();

        try_stream! {
            let mikan_base_url = ctx.mikan().base_url().clone();
            let rss_url = build_mikan_bangumi_subscription_rss_url(mikan_base_url.clone(), &mikan_bangumi_id, Some(&mikan_fansub_id));
            let bytes = fetch_bytes(ctx.mikan(), rss_url).await?;

            let channel = rss::Channel::read_from(&bytes[..])?;

            for (idx, item) in channel.items.into_iter().enumerate() {
                let item = MikanRssItem::try_from(item).inspect_err(
                    |error| tracing::warn!(error = %error, "failed to extract rss item idx = {}", idx),
                )?;
                yield item
            }
        }
    }

    pub fn try_from_model(model: &subscriptions::Model) -> RecorderResult<Self> {
        let source_url = Url::parse(&model.source_url)?;

        let meta = MikanBangumiRssUrlMeta::from_url(&source_url)
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

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use downloader::bittorrent::BITTORRENT_MIME_TYPE;
    use rstest::rstest;
    use url::Url;

    use crate::{
        errors::RecorderResult,
        extract::mikan::{
            MikanBangumiIndexRssChannel, MikanBangumiRssChannel, MikanRssChannel,
            extract_mikan_rss_channel_from_rss_link,
        },
        test_utils::mikan::build_testing_mikan_client,
    };

    #[rstest]
    #[tokio::test]
    async fn test_parse_mikan_rss_channel_from_rss_link() -> RecorderResult<()> {
        let mut mikan_server = mockito::Server::new_async().await;

        let mikan_base_url = Url::parse(&mikan_server.url())?;

        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;

        {
            let bangumi_rss_url =
                mikan_base_url.join("/RSS/Bangumi?bangumiId=3141&subgroupid=370")?;

            let bangumi_rss_mock = mikan_server
                .mock("GET", bangumi_rss_url.path())
                .with_body_from_file("tests/resources/mikan/Bangumi-3141-370.rss")
                .match_query(mockito::Matcher::Any)
                .create_async()
                .await;

            let channel = scrape_mikan_rss_channel_from_rss_link(&mikan_client, bangumi_rss_url)
                .await
                .expect("should get mikan channel from rss url");

            assert_matches!(
                &channel,
                MikanRssChannel::Bangumi(MikanBangumiRssChannel { .. })
            );

            assert_matches!(&channel.name(), Some("葬送的芙莉莲"));

            let items = channel.items();
            let first_sub_item = items
                .first()
                .expect("mikan subscriptions should have at least one subs");

            assert_eq!(first_sub_item.mime, BITTORRENT_MIME_TYPE);

            assert!(
                &first_sub_item
                    .homepage
                    .as_str()
                    .starts_with("https://mikanani.me/Home/Episode")
            );

            let name = first_sub_item.title.as_str();
            assert!(name.contains("葬送的芙莉莲"));

            bangumi_rss_mock.expect(1);
        }
        {
            let bangumi_rss_url = mikan_base_url.join("/RSS/Bangumi?bangumiId=3416")?;

            let bangumi_rss_mock = mikan_server
                .mock("GET", bangumi_rss_url.path())
                .match_query(mockito::Matcher::Any)
                .with_body_from_file("tests/resources/mikan/Bangumi-3416.rss")
                .create_async()
                .await;

            let channel = scrape_mikan_rss_channel_from_rss_link(&mikan_client, bangumi_rss_url)
                .await
                .expect("should get mikan channel from rss url");

            assert_matches!(
                &channel,
                MikanRssChannel::BangumiIndex(MikanBangumiIndexRssChannel { .. })
            );

            assert_matches!(&channel.name(), Some("叹气的亡灵想隐退"));

            bangumi_rss_mock.expect(1);
        }
        Ok(())
    }
}
