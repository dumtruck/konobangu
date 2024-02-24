use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{
    Bangumi, CustomSchemaManagerExt, Episodes, GeneralIds, Subscribers, Subscriptions,
};
use crate::models::{subscribers::ROOT_SUBSCRIBER, subscriptions};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt)
            .await?;
        manager
            .create_table(
                table_auto(Subscribers::Table)
                    .col(pk_auto(Subscribers::Id))
                    .col(string_len_uniq(Subscribers::Pid, 64))
                    .col(string(Subscribers::DisplayName))
                    .to_owned(),
            )
            .await?;
        manager
            .create_postgres_auto_update_ts_trigger_for_col(
                Subscribers::Table,
                GeneralIds::UpdatedAt,
            )
            .await?;

        let insert = Query::insert()
            .into_table(Subscribers::Table)
            .columns([Subscribers::Pid, Subscribers::DisplayName])
            .values_panic([ROOT_SUBSCRIBER.into(), ROOT_SUBSCRIBER.into()])
            .to_owned();
        manager.exec_stmt(insert).await?;

        manager
            .create_postgres_enum_for_active_enum(
                subscriptions::SubscriptionCategoryEnum,
                &[
                    subscriptions::SubscriptionCategory::Mikan,
                    subscriptions::SubscriptionCategory::Manual,
                    subscriptions::SubscriptionCategory::Bangumi,
                ],
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
                    .col(enumeration(
                        Subscriptions::Category,
                        subscriptions::SubscriptionCategoryEnum,
                        subscriptions::SubscriptionCategory::iden_values(),
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscription_subscriber_id")
                            .from(Subscriptions::Table, Subscriptions::SubscriberId)
                            .to(Subscribers::Table, Subscribers::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(
                Subscriptions::Table,
                GeneralIds::UpdatedAt,
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
                            .name("fk_bangumi_subscription_id")
                            .from(Bangumi::Table, Bangumi::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(Bangumi::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .create_table(
                table_auto(Episodes::Table)
                    .col(pk_auto(Episodes::Id))
                    .col(text(Episodes::DisplayName))
                    .col(integer(Episodes::BangumiId))
                    .col(text(Episodes::OutputName))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episode_bangumi_id")
                            .from(Episodes::Table, Episodes::BangumiId)
                            .to(Bangumi::Table, Bangumi::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(Episodes::Table, GeneralIds::UpdatedAt)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Episodes::Table, GeneralIds::UpdatedAt)
            .await?;
        manager
            .drop_table(Table::drop().table(Episodes::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Bangumi::Table, GeneralIds::UpdatedAt)
            .await?;
        manager
            .drop_table(Table::drop().table(Bangumi::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(
                Subscriptions::Table,
                GeneralIds::UpdatedAt,
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Subscribers::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Subscribers::Table).to_owned())
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(subscriptions::SubscriptionCategoryEnum)
            .await?;

        Ok(())
    }
}
