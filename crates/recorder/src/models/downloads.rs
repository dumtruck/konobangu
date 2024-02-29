use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, Condition, QueryOrder, QuerySelect};

pub use crate::models::entities::downloads::*;
use crate::{
    models::subscriptions::{self, SubscriptionCategory},
    subscriptions::mikan::{MikanSubscriptionEngine, MikanSubscriptionItem},
};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn from_mikan_subscription_item(m: MikanSubscriptionItem, subscription_id: i32) -> Self {
        Self {
            origin_name: ActiveValue::Set(m.title.clone()),
            display_name: ActiveValue::Set(m.title),
            subscription_id: ActiveValue::Set(subscription_id),
            status: ActiveValue::Set(DownloadStatus::Pending),
            mime: ActiveValue::Set(DownloadMime::BitTorrent),
            url: ActiveValue::Set(m.url),
            curr_size: ActiveValue::Set(m.content_length.as_ref().map(|_| 0)),
            all_size: ActiveValue::Set(m.content_length),
            homepage: ActiveValue::Set(m.homepage),
            ..Default::default()
        }
    }
}

impl Model {
    pub async fn pull_subscription(
        db: &DatabaseConnection,
        item: &subscriptions::Model,
    ) -> eyre::Result<Vec<i32>> {
        match &item.category {
            SubscriptionCategory::Mikan => {
                let items =
                    MikanSubscriptionEngine::subscription_items_from_rss_url(&item.source_url)
                        .await?;
                let all_items = items.collect::<Vec<_>>();

                let last_old_id = {
                    Entity::find()
                        .select_only()
                        .column(Column::Id)
                        .order_by_desc(Column::Id)
                        .filter(Column::SubscriptionId.eq(item.id))
                        .one(db)
                        .await?
                }
                .map(|i| i.id);

                if all_items.is_empty() {
                    return Ok(vec![]);
                }

                let new_items = all_items
                    .into_iter()
                    .map(|i| ActiveModel::from_mikan_subscription_item(i, item.id));

                let insert_result = Entity::insert_many(new_items)
                    .on_conflict(OnConflict::column(Column::Url).do_nothing().to_owned())
                    .exec(db)
                    .await?;

                let insert_ids = Entity::find()
                    .select_only()
                    .column(Column::Id)
                    .filter({
                        let mut cond = Condition::all()
                            .add(Column::SubscriptionId.eq(item.id))
                            .add(Column::Id.lte(insert_result.last_insert_id));

                        if let Some(last_old_id) = last_old_id {
                            cond = cond.add(Column::Id.gt(last_old_id))
                        }

                        cond
                    })
                    .all(db)
                    .await?;

                Ok(insert_ids.into_iter().map(|i| i.id).collect::<Vec<_>>())
            }
            _ => {
                todo!("other subscription categories")
            }
        }
    }
}
