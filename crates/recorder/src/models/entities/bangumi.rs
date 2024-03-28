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
    pub season: u32,
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
    pub season: u32,
    pub filter: Option<BangumiFilter>,
    pub poster_link: Option<String>,
    pub save_path: Option<String>,
    pub last_ep: u32,
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
