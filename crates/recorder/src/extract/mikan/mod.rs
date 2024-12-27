pub mod client;
pub mod config;
pub mod constants;
pub mod rss_parser;
pub mod web_parser;

pub use client::{AppMikanClient, AppMikanClientInitializer};
pub use config::{AppMikanConfig, MIKAN_CONF_KEY};
pub use constants::{MIKAN_BASE_URL, MIKAN_BUCKET_KEY};
pub use rss_parser::{
    build_mikan_bangumi_rss_link, build_mikan_subscriber_aggregation_rss_link,
    parse_mikan_bangumi_id_from_rss_link, parse_mikan_rss_channel_from_rss_link,
    parse_mikan_rss_items_from_rss_link, parse_mikan_subscriber_aggregation_id_from_rss_link,
    MikanBangumiAggregationRssChannel, MikanBangumiRssChannel, MikanBangumiRssLink,
    MikanRssChannel, MikanRssItem, MikanSubscriberAggregationRssChannel,
    MikanSubscriberAggregationRssLink,
};
pub use web_parser::{
    build_mikan_bangumi_homepage, build_mikan_episode_homepage,
    parse_mikan_bangumi_meta_from_mikan_homepage, parse_mikan_episode_meta_from_mikan_homepage,
    MikanBangumiMeta, MikanEpisodeMeta,
};
