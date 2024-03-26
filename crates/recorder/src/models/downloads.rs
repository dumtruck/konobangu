use itertools::Itertools;
use loco_rs::app::AppContext;
use sea_orm::{
    prelude::*,
    sea_query::{InsertStatement, OnConflict},
};

pub use crate::models::entities::downloads::*;
use crate::{
    models::{
        db_utils::insert_many_with_returning_all,
        subscriptions::{self, SubscriptionCategory},
    },
    parsers::mikan::{
        mikan_client::MikanClient, parse_mikan_rss_items_from_rss_link, MikanRssItem,
    },
};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_mikan_rss_item(m: MikanRssItem, subscription_id: i32) -> Self {
        todo!()
    }
}

impl Model {
    pub async fn pull_subscription(
        ctx: AppContext,
        subscription: &subscriptions::Model,
    ) -> eyre::Result<Vec<Model>> {
        let db = &ctx.db;
        match &subscription.category {
            SubscriptionCategory::Mikan => {
                let subscriber_id = subscription.subscriber_id;
                let client = MikanClient::new(subscriber_id).await?;
                let items =
                    parse_mikan_rss_items_from_rss_link(&client, &subscription.source_url).await?;
                let all_items = items.collect::<Vec<_>>();

                if all_items.is_empty() {
                    return Ok(vec![]);
                }

                let new_items = all_items
                    .into_iter()
                    .map(|i| ActiveModel::from_mikan_rss_item(i, subscription.id))
                    .collect_vec();

                // insert and filter out duplicated items
                let new_items: Vec<Model> =
                    insert_many_with_returning_all(db, new_items, |stat: &mut InsertStatement| {
                        stat.on_conflict(OnConflict::column(Column::Url).do_nothing().to_owned());
                    })
                    .await?;

                Ok(new_items)
            }
            _ => {
                todo!("other subscription categories")
            }
        }
    }
}
