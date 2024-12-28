use loco_rs::schema::jsonb_null;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{
    Bangumi, CustomSchemaManagerExt, Episodes, GeneralIds, Subscribers, Subscriptions,
};
use crate::models::{
    subscribers::ROOT_SUBSCRIBER,
    subscriptions::{self, SubscriptionCategoryEnum},
};

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
                    .col(jsonb_null(Subscribers::BangumiConf))
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

        create_postgres_enum_for_active_enum!(
            manager,
            subscriptions::SubscriptionCategoryEnum,
            subscriptions::SubscriptionCategory::Mikan,
            subscriptions::SubscriptionCategory::Manual
        )
        .await?;

        manager
            .create_table(
                table_auto(Subscriptions::Table)
                    .col(pk_auto(Subscriptions::Id))
                    .col(string(Subscriptions::DisplayName))
                    .col(integer(Subscriptions::SubscriberId))
                    .col(text(Subscriptions::SourceUrl))
                    .col(boolean(Subscriptions::Enabled))
                    .col(enumeration(
                        Subscriptions::Category,
                        subscriptions::SubscriptionCategoryEnum,
                        subscriptions::SubscriptionCategory::iden_values(),
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriptions_subscriber_id")
                            .from(Subscriptions::Table, Subscriptions::SubscriberId)
                            .to(Subscribers::Table, Subscribers::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
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
                    .col(text_null(Bangumi::MikanBangumiId))
                    .col(integer(Bangumi::SubscriptionId))
                    .col(integer(Bangumi::SubscriberId))
                    .col(text(Bangumi::DisplayName))
                    .col(text(Bangumi::RawName))
                    .col(integer(Bangumi::Season))
                    .col(text_null(Bangumi::SeasonRaw))
                    .col(text_null(Bangumi::Fansub))
                    .col(text_null(Bangumi::MikanFansubId))
                    .col(jsonb_null(Bangumi::Filter))
                    .col(text_null(Bangumi::RssLink))
                    .col(text_null(Bangumi::PosterLink))
                    .col(text_null(Bangumi::SavePath))
                    .col(boolean(Bangumi::Deleted).default(false))
                    .col(text_null(Bangumi::Homepage))
                    .col(jsonb_null(Bangumi::Extra))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bangumi_subscription_id")
                            .from(Bangumi::Table, Bangumi::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bangumi_subscriber_id")
                            .from(Bangumi::Table, Bangumi::SubscriberId)
                            .to(Subscribers::Table, Subscribers::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_bangumi_mikan_bangumi_id")
                    .table(Bangumi::Table)
                    .col(Bangumi::MikanBangumiId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_bangumi_mikan_fansub_id")
                    .table(Bangumi::Table)
                    .col(Bangumi::MikanFansubId)
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
                    .col(text_null(Episodes::MikanEpisodeId))
                    .col(text(Episodes::RawName))
                    .col(text(Episodes::DisplayName))
                    .col(integer(Episodes::BangumiId))
                    .col(integer(Episodes::SubscriptionId))
                    .col(integer(Episodes::SubscriberId))
                    .col(text_null(Episodes::SavePath))
                    .col(text_null(Episodes::Resolution))
                    .col(integer(Episodes::Season))
                    .col(text_null(Episodes::SeasonRaw))
                    .col(text_null(Episodes::Fansub))
                    .col(text_null(Episodes::PosterLink))
                    .col(integer(Episodes::EpisodeIndex))
                    .col(text_null(Episodes::Homepage))
                    .col(text_null(Episodes::Subtitle))
                    .col(boolean(Episodes::Deleted).default(false))
                    .col(text_null(Episodes::Source))
                    .col(jsonb_null(Episodes::Extra))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episodes_subscription_id")
                            .from(Episodes::Table, Episodes::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episodes_bangumi_id")
                            .from(Episodes::Table, Episodes::BangumiId)
                            .to(Bangumi::Table, Bangumi::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episodes_subscriber_id")
                            .from(Episodes::Table, Episodes::SubscriberId)
                            .to(Subscribers::Table, Subscribers::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_episodes_mikan_episode_id")
                    .table(Episodes::Table)
                    .col(Episodes::MikanEpisodeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_episodes_bangumi_id_mikan_episode_id")
                    .table(Episodes::Table)
                    .col(Episodes::BangumiId)
                    .col(Episodes::MikanEpisodeId)
                    .unique()
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
            .drop_table(Table::drop().table(Episodes::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Episodes::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Bangumi::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Bangumi::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(
                Subscriptions::Table,
                GeneralIds::UpdatedAt,
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Subscribers::Table).to_owned())
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Subscribers::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(subscriptions::SubscriptionCategoryEnum)
            .await?;

        manager
            .drop_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(SubscriptionCategoryEnum)
            .await?;

        Ok(())
    }
}
