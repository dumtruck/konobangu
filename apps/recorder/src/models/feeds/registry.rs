use rss::Channel;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use url::Url;

use crate::{
    app::AppContextTrait,
    errors::{RecorderError, RecorderResult},
    models::{
        episodes,
        feeds::{self, FeedSource, RssFeedTrait, SubscriptionEpisodesFeed},
        subscriptions,
    },
};

pub enum Feed {
    SubscritpionEpisodes(SubscriptionEpisodesFeed),
}

impl Feed {
    pub async fn from_model(ctx: &dyn AppContextTrait, m: feeds::Model) -> RecorderResult<Self> {
        match m.feed_source {
            FeedSource::SubscriptionEpisode => {
                let db = ctx.db();
                let (subscription, episodes) = if let Some(subscription_id) = m.subscription_id
                    && let Some((subscription, episodes)) = subscriptions::Entity::find()
                        .filter(subscriptions::Column::Id.eq(subscription_id))
                        .find_with_related(episodes::Entity)
                        .all(db)
                        .await?
                        .pop()
                {
                    (subscription, episodes)
                } else {
                    return Err(RecorderError::ModelEntityNotFound {
                        entity: "Subscription".into(),
                    });
                };
                Ok(Feed::SubscritpionEpisodes(
                    SubscriptionEpisodesFeed::from_model(m, subscription, episodes),
                ))
            }
        }
    }

    pub fn into_rss_channel(
        self,
        ctx: &dyn AppContextTrait,
        api_base: &Url,
    ) -> RecorderResult<Channel> {
        match self {
            Self::SubscritpionEpisodes(feed) => feed.into_channel(ctx, api_base),
        }
    }
}
