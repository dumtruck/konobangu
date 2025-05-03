mod client;
mod config;
mod constants;
mod credential;
mod rss;
mod web;

pub use client::MikanClient;
pub use config::MikanConfig;
pub use constants::{
    MIKAN_ACCOUNT_MANAGE_PAGE_PATH, MIKAN_LOGIN_PAGE_PATH, MIKAN_LOGIN_PAGE_SEARCH,
    MIKAN_POSTER_BUCKET_KEY, MIKAN_UNKNOWN_FANSUB_ID, MIKAN_UNKNOWN_FANSUB_NAME,
};
pub use credential::MikanCredentialForm;
pub use rss::{
    MikanBangumiIndexRssChannel, MikanBangumiRssChannel, MikanBangumiRssUrlMeta, MikanRssChannel,
    MikanRssItem, MikanSubscriberAggregationRssUrlMeta, MikanSubscriberStreamRssChannel,
    build_mikan_bangumi_rss_url, build_mikan_subscriber_aggregation_rss_url,
    extract_mikan_bangumi_id_from_rss_url, extract_mikan_rss_channel_from_rss_link,
    extract_mikan_subscriber_aggregation_id_from_rss_link,
};
pub use web::{
    MikanBangumiHomepageUrlMeta, MikanBangumiIndexHomepageUrlMeta, MikanBangumiIndexMeta,
    MikanBangumiMeta, MikanBangumiPosterMeta, MikanEpisodeHomepageUrlMeta, MikanEpisodeMeta,
    MikanSeasonFlowUrlMeta, MikanSeasonStr, build_mikan_bangumi_expand_subscribed_url,
    build_mikan_bangumi_homepage_url, build_mikan_episode_homepage_url,
    build_mikan_season_flow_url, extract_mikan_bangumi_index_meta_list_from_season_flow_fragment,
    extract_mikan_episode_meta_from_episode_homepage_html,
    scrape_mikan_bangumi_meta_from_bangumi_homepage_url,
    scrape_mikan_bangumi_meta_list_from_season_flow_url,
    scrape_mikan_episode_meta_from_episode_homepage_url, scrape_mikan_poster_data_from_image_url,
    scrape_mikan_poster_meta_from_image_url,
};
