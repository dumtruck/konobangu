use std::collections::HashMap;
use sea_orm::{prelude::*, ActiveValue, Condition, QuerySelect, SelectColumns};

use crate::models::_entities::downloads::*;
use crate::models::prelude::{SubscriptionCategory, subscriptions};
use crate::subscriptions::mikan::MikanSubscriptionEngine;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn pull_subscription(
        db: &DatabaseConnection,
        item: &subscriptions::Model,
    ) -> eyre::Result<()> {
        match &item.category {
            SubscriptionCategory::Mikan => {
                let items =
                    MikanSubscriptionEngine::subscription_items_from_rss_url(&item.source_url).
                        await?;
                let items = items.collect::<Vec<_>>();
                let mut all_items = items
                    .into_iter()
                    .map(|item| (item.url.clone(), item))
                    .collect::<HashMap<_, _>>();

                let existed_items = {
                    Entity::find()
                        .filter(
                            Condition::all()
                                .add(Column::SubscriptionId.eq(item.id))
                                .add(Column::Url.is_in(all_items.keys().cloned()))
                        )
                        .select_only()
                        .select_column(Column::Url)
                        .all(db).await?
                };

                for dl in existed_items {
                    all_items.remove(&dl.url as &str);
                }

                let new_items = all_items.into_values().map(|i| {
                    ActiveModel {
                        origin_name: ActiveValue::Set(i.title.clone()),
                        display_name: ActiveValue::Set(i.title),
                        subscription_id: ActiveValue::Set(item.id),
                        status: ActiveValue::Set(DownloadStatus::Pending),
                        mime: ActiveValue::Set(DownloadMime::BitTorrent),
                        url: ActiveValue::Set(i.url),
                        all_size: ActiveValue::Set(i.content_length.unwrap_or_default()),
                        curr_size: ActiveValue::Set(0),
                        ..Default::default()
                    }
                });

                Entity::insert_many(new_items)
                    .exec(db)
                    .await?;
            }
            _ => {
                todo!("other subscription categories")
            }
        }
        Ok(())
    }
}
