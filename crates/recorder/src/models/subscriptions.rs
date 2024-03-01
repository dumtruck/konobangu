use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

pub use super::entities::subscriptions::{self, *};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubscriptionCreateFromRssDto {
    pub rss_link: String,
    pub display_name: String,
    pub aggregate: bool,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "category")]
pub enum SubscriptionCreateDto {
    Mikan(SubscriptionCreateFromRssDto),
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_create_dto(create_dto: SubscriptionCreateDto, subscriber_id: i32) -> Self {
        match create_dto {
            SubscriptionCreateDto::Mikan(create_dto) => {
                Self::from_rss_create_dto(SubscriptionCategory::Mikan, create_dto, subscriber_id)
            }
        }
    }

    fn from_rss_create_dto(
        category: SubscriptionCategory,
        create_dto: SubscriptionCreateFromRssDto,
        subscriber_id: i32,
    ) -> Self {
        Self {
            display_name: ActiveValue::Set(create_dto.display_name),
            enabled: ActiveValue::Set(create_dto.enabled.unwrap_or(false)),
            aggregate: ActiveValue::Set(create_dto.aggregate),
            subscriber_id: ActiveValue::Set(subscriber_id),
            category: ActiveValue::Set(category),
            source_url: ActiveValue::Set(create_dto.rss_link),
            ..Default::default()
        }
    }
}

impl Model {
    pub async fn add_subscription(
        db: &DatabaseConnection,
        create_dto: SubscriptionCreateDto,
        subscriber_id: i32,
    ) -> eyre::Result<Self> {
        let subscription = ActiveModel::from_create_dto(create_dto, subscriber_id);

        Ok(subscription.insert(db).await?)
    }

    pub async fn toggle_iters(
        db: &DatabaseConnection,
        ids: impl Iterator<Item = i32>,
        enabled: bool,
    ) -> eyre::Result<()> {
        Entity::update_many()
            .col_expr(Column::Enabled, Expr::value(enabled))
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_iters(
        db: &DatabaseConnection,
        ids: impl Iterator<Item = i32>,
    ) -> eyre::Result<()> {
        Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(())
    }
}
