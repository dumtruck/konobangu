use sea_orm::{entity::prelude::*, ActiveValue};

pub use super::entities::subscriptions::{self, *};
use crate::subscriptions::defs::RssCreateDto;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn add_rss(
        db: &DatabaseConnection,
        create_dto: RssCreateDto,
        subscriber_id: i32,
    ) -> eyre::Result<Self> {
        let subscription = ActiveModel {
            display_name: ActiveValue::Set(create_dto.display_name),
            enabled: ActiveValue::Set(create_dto.enabled.unwrap_or(false)),
            aggregate: ActiveValue::Set(create_dto.aggregate),
            subscriber_id: ActiveValue::Set(subscriber_id),
            category: ActiveValue::Set(SubscriptionCategory::Mikan),
            source_url: ActiveValue::Set(create_dto.rss_link),
            ..Default::default()
        };

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
