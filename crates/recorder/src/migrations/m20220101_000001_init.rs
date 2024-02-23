use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{Bangumi, Episodes, Subscribers, Subscriptions};
use crate::models::subscribers::ROOT_SUBSCRIBER;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Subscribers::Table)
                    .col(pk_auto(Subscribers::Id))
                    .col(string_len_uniq(Subscribers::Pid, 64))
                    .col(string(Subscribers::DisplayName))
                    .to_owned(),
            )
            .await?;

        let insert = Query::insert()
            .into_table(Subscribers::Table)
            .columns([Subscribers::Pid, Subscribers::DisplayName])
            .values_panic([ROOT_SUBSCRIBER.into(), ROOT_SUBSCRIBER.into()])
            .to_owned();
        manager.exec_stmt(insert).await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("subscription_category"))
                    .values([
                        Alias::new("mikan"),
                        Alias::new("manual"),
                        Alias::new("bangumi"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                table_auto(Subscriptions::Table)
                    .col(pk_auto(Subscriptions::Id))
                    .col(string(Subscriptions::DisplayName))
                    .col(integer(Subscriptions::SubscriberId))
                    .col(text(Subscriptions::SourceUrl))
                    .col(boolean(Subscriptions::Aggregate))
                    .col(boolean(Subscriptions::Enabled))
                    .foreign_key(
                        ForeignKey::create()
                            .name("subscription_subscriber_id")
                            .from(Subscriptions::Table, Subscriptions::SubscriberId)
                            .to(Subscribers::Table, Subscribers::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                table_auto(Bangumi::Table)
                    .col(pk_auto(Bangumi::Id))
                    .col(text(Bangumi::DisplayName))
                    .col(integer(Bangumi::SubscriptionId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("bangumi_subscription_id")
                            .from(Bangumi::Table, Bangumi::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                table_auto(Episodes::Table)
                    .col(pk_auto(Episodes::Id))
                    .col(text(Episodes::DisplayName))
                    .col(integer(Episodes::BangumiId))
                    .col(text(Episodes::DownloadUrl))
                    .col(tiny_integer(Episodes::DownloadProgress).default(0))
                    .col(text(Episodes::OutputName))
                    .foreign_key(
                        ForeignKey::create()
                            .name("episode_bangumi_id")
                            .from(Episodes::Table, Episodes::BangumiId)
                            .to(Bangumi::Table, Bangumi::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Episodes::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Bangumi::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("subscription_category"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Subscribers::Table).to_owned())
            .await
    }
}
