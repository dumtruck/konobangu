pub mod client;
pub mod config;
pub mod constants;
pub mod rss_extract;
pub mod web_extract;

pub use client::{AppMikanClient, AppMikanClientInitializer};
pub use config::AppMikanConfig;
pub use constants::MIKAN_BUCKET_KEY;
pub use rss_extract::{
    MikanBangumiAggregationRssChannel, MikanBangumiRssChannel, MikanBangumiRssLink,
    MikanRssChannel, MikanRssItem, MikanSubscriberAggregationRssChannel,
    MikanSubscriberAggregationRssLink, build_mikan_bangumi_rss_link,
    build_mikan_subscriber_aggregation_rss_link, extract_mikan_bangumi_id_from_rss_link,
    extract_mikan_rss_channel_from_rss_link, extract_mikan_subscriber_aggregation_id_from_rss_link,
};
pub use web_extract::{
    MikanBangumiMeta, MikanEpisodeMeta, build_mikan_bangumi_homepage, build_mikan_episode_homepage,
    extract_mikan_bangumi_meta_from_bangumi_homepage,
    extract_mikan_episode_meta_from_episode_homepage,
};
