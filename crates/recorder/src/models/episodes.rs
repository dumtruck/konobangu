use std::sync::Arc;

use loco_rs::app::AppContext;
use sea_orm::{entity::prelude::*, sea_query::OnConflict, ActiveValue};

use super::bangumi;
pub use super::entities::episodes::*;
use crate::{
    app::AppContextExt,
    extract::{
        mikan::{build_mikan_episode_homepage, MikanEpisodeMeta},
        rawname::parse_episode_meta_from_raw_name,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct MikanEpsiodeCreation {
    pub episode: MikanEpisodeMeta,
    pub bangumi: Arc<bangumi::Model>,
}

impl Model {
    pub async fn add_episodes(
        ctx: &AppContext,
        creations: impl IntoIterator<Item = MikanEpsiodeCreation>,
    ) -> eyre::Result<()> {
        let db = &ctx.db;
        let new_episode_active_modes = creations
            .into_iter()
            .flat_map(|cr| ActiveModel::from_mikan_episode_meta(ctx, cr));

        Entity::insert_many(new_episode_active_modes)
            .on_conflict(
                OnConflict::columns([Column::BangumiId, Column::MikanEpisodeId])
                    .do_nothing()
                    .to_owned(),
            )
            .on_empty_do_nothing()
            .exec(db)
            .await?;

        Ok(())
    }
}

impl ActiveModel {
    pub fn from_mikan_episode_meta(
        ctx: &AppContext,
        creation: MikanEpsiodeCreation,
    ) -> eyre::Result<Self> {
        let item = creation.episode;
        let bgm = creation.bangumi;
        let raw_meta = parse_episode_meta_from_raw_name(&item.episode_title)?;
        let homepage = build_mikan_episode_homepage(
            ctx.get_mikan_client().base_url(),
            &item.mikan_episode_id,
        )?;

        Ok(Self {
            mikan_episode_id: ActiveValue::Set(Some(item.mikan_episode_id)),
            raw_name: ActiveValue::Set(item.episode_title.clone()),
            display_name: ActiveValue::Set(item.episode_title.clone()),
            bangumi_id: ActiveValue::Set(bgm.id),
            subscription_id: ActiveValue::Set(bgm.subscription_id),
            subscriber_id: ActiveValue::Set(bgm.subscriber_id),
            resolution: ActiveValue::Set(raw_meta.resolution),
            season: ActiveValue::Set(if raw_meta.season > 0 {
                raw_meta.season
            } else {
                bgm.season
            }),
            season_raw: ActiveValue::Set(raw_meta.season_raw.or_else(|| bgm.season_raw.clone())),
            fansub: ActiveValue::Set(raw_meta.fansub.or_else(|| bgm.fansub.clone())),
            poster_link: ActiveValue::Set(bgm.poster_link.clone()),
            episode_index: ActiveValue::Set(raw_meta.episode_index),
            homepage: ActiveValue::Set(Some(homepage.to_string())),
            subtitle: ActiveValue::Set(raw_meta.subtitle.map(|s| vec![s])),
            source: ActiveValue::Set(raw_meta.source),
            extra: ActiveValue::Set(EpisodeExtra {
                name_zh: raw_meta.name_zh,
                name_en: raw_meta.name_en,
                name_jp: raw_meta.name_jp,
                s_name_en: raw_meta.name_en_no_season,
                s_name_jp: raw_meta.name_jp_no_season,
                s_name_zh: raw_meta.name_zh_no_season,
            }),
            ..Default::default()
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}
