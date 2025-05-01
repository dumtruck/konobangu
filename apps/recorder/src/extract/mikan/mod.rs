pub mod client;
pub mod config;
pub mod constants;
pub mod rss_extract;
pub mod web_extract;

pub use client::{MikanClient, MikanCredentialForm};
pub use config::MikanConfig;
pub use constants::MIKAN_BUCKET_KEY;
pub use rss_extract::{
    MikanBangumiAggregationRssChannel, MikanBangumiRssChannel, MikanBangumiRssUrlMeta,
    MikanRssChannel, MikanRssItem, MikanSubscriberAggregationRssChannel,
    MikanSubscriberAggregationRssUrlMeta, build_mikan_bangumi_rss_url,
    build_mikan_subscriber_aggregation_rss_url, extract_mikan_bangumi_id_from_rss_url,
    extract_mikan_rss_channel_from_rss_link, extract_mikan_subscriber_aggregation_id_from_rss_link,
};
pub use web_extract::{
    MikanBangumiMeta, MikanEpisodeMeta, MikanSeasonStr, build_mikan_bangumi_homepage_url,
    build_mikan_episode_homepage_url, build_mikan_season_flow_url,
    extract_mikan_bangumi_indices_meta_from_season_flow_fragment,
    extract_mikan_bangumi_meta_from_bangumi_homepage,
    extract_mikan_episode_meta_from_episode_homepage,
};
