use std::collections::HashSet;

use itertools::Itertools;
use regex::Regex;
use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, DeriveDisplay,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "bangumi_distribution"
)]
#[serde(rename_all = "snake_case")]
pub enum BangumiDistribution {
    #[sea_orm(string_value = "movie")]
    Movie,
    #[sea_orm(string_value = "ova")]
    Ova,
    #[sea_orm(string_value = "oad")]
    Oad,
    #[sea_orm(string_value = "sp")]
    Sp,
    #[sea_orm(string_value = "ex")]
    Ex,
    #[sea_orm(string_value = "tv")]
    Tv,
    #[sea_orm(string_value = "unknown")]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "snake_case")]
pub enum BangumiRenameMethod {
    Pn,
    Advance,
    SubtitlePn,
    SubtitleAdvance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct SubscribeBangumiConfigOverride {
    pub leading_fansub_tag: Option<bool>,
    pub complete_history_episodes: Option<bool>,
    pub rename_method: Option<BangumiRenameMethod>,
    pub remove_bad_torrent: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct BangumiFilter {
    pub plaintext_filters: Option<Vec<String>>,
    pub regex_filters: Option<Vec<String>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BangumiUniqueKey {
    pub official_title: String,
    pub season: i32,
    pub fansub: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "bangumi")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub subscription_id: i32,
    pub display_name: String,
    pub official_title: String,
    pub fansub: Option<String>,
    pub season: i32,
    pub filter: Option<BangumiFilter>,
    pub poster_link: Option<String>,
    pub save_path: Option<String>,
    pub last_ep: i32,
    pub bangumi_conf_override: Option<SubscribeBangumiConfigOverride>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscriptions::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscriptions::Column::Id"
    )]
    Subscription,
    #[sea_orm(has_many = "super::episodes::Entity")]
    Episode,
}

impl Related<super::episodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Episode.def()
    }
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl BangumiFilter {
    pub fn is_match(&self, title: &str) -> eyre::Result<bool> {
        if let Some(regex_filters) = &self.regex_filters {
            let combined_regex = Regex::new(&regex_filters.join("|"))?;
            if combined_regex.is_match(title) {
                return Ok(true);
            }
        } else if let Some(plain_filters) = &self.plaintext_filters {
            for f in plain_filters {
                if title.contains(f) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Model {
    pub fn get_unique_key(&self) -> BangumiUniqueKey {
        BangumiUniqueKey {
            official_title: self.official_title.clone(),
            season: self.season,
            fansub: self.fansub.clone(),
        }
    }

    pub async fn find_by_unique_keys(
        db: &DatabaseConnection,
        unique_keys: impl Iterator<Item = &BangumiUniqueKey>,
    ) -> eyre::Result<Vec<Self>> {
        let unique_keys = unique_keys.collect::<HashSet<_>>();
        let mut found = Entity::find()
            .filter(Column::OfficialTitle.is_in(unique_keys.iter().map(|k| &k.official_title)))
            .all(db)
            .await?;

        found = found
            .into_iter()
            .filter(|m| unique_keys.contains(&m.get_unique_key()))
            .collect_vec();

        Ok(found)
    }
}
