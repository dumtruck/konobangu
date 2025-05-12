mod client;
mod config;
mod constants;
mod credential;
mod subscription;
mod web;

pub use client::MikanClient;
pub use config::MikanConfig;
pub use constants::{
    MIKAN_ACCOUNT_MANAGE_PAGE_PATH, MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH,
    MIKAN_LOGIN_PAGE_PATH, MIKAN_LOGIN_PAGE_SEARCH, MIKAN_POSTER_BUCKET_KEY,
    MIKAN_SEASON_FLOW_PAGE_PATH, MIKAN_UNKNOWN_FANSUB_ID, MIKAN_UNKNOWN_FANSUB_NAME,
};
pub use credential::MikanCredentialForm;
pub use subscription::{
    MikanBangumiSubscription, MikanSeasonSubscription, MikanSubscriberSubscription,
};
pub use web::{
    MikanBangumiHash, MikanBangumiIndexHash, MikanBangumiIndexMeta, MikanBangumiMeta,
    MikanBangumiPosterMeta, MikanEpisodeHash, MikanEpisodeMeta, MikanRssItem,
    MikanSeasonFlowUrlMeta, MikanSeasonStr, MikanSubscriberSubscriptionRssUrlMeta,
    build_mikan_bangumi_expand_subscribed_url, build_mikan_bangumi_homepage_url,
    build_mikan_bangumi_subscription_rss_url, build_mikan_episode_homepage_url,
    build_mikan_season_flow_url, build_mikan_subscriber_subscription_rss_url,
    extract_mikan_bangumi_index_meta_list_from_season_flow_fragment,
    extract_mikan_bangumi_meta_from_expand_subscribed_fragment,
    extract_mikan_episode_meta_from_episode_homepage_html,
    scrape_mikan_bangumi_meta_from_bangumi_homepage_url,
    scrape_mikan_bangumi_meta_list_from_season_flow_url,
    scrape_mikan_bangumi_meta_stream_from_season_flow_url,
    scrape_mikan_episode_meta_from_episode_homepage_url, scrape_mikan_poster_data_from_image_url,
    scrape_mikan_poster_meta_from_image_url,
};
