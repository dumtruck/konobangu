use loco_rs::schema::jsonb_null;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::{
    Bangumi, CustomSchemaManagerExt, Episodes, GeneralIds, Subscribers, Subscriptions,
};
use crate::models::{
    subscribers::{ROOT_SUBSCRIBER_ID, ROOT_SUBSCRIBER_NAME},
    subscriptions,
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
            .columns([Subscribers::Id, Subscribers::Pid, Subscribers::DisplayName])
            .values_panic([
                ROOT_SUBSCRIBER_ID.into(),
                ROOT_SUBSCRIBER_NAME.into(),
                ROOT_SUBSCRIBER_NAME.into(),
            ])
            .to_owned();
        manager.exec_stmt(insert).await?;

        manager
            .create_postgres_enum_for_active_enum(
                subscriptions::SubscriptionCategoryEnum,
                &[
                    subscriptions::SubscriptionCategory::Mikan,
                    subscriptions::SubscriptionCategory::Tmdb,
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
                    .col(integer(Bangumi::SubscriptionId))
                    .col(text(Bangumi::DisplayName))
                    .col(text(Bangumi::OfficialTitle))
                    .col(string_null(Bangumi::Fansub))
                    .col(unsigned(Bangumi::Season))
                    .col(jsonb_null(Bangumi::Filter))
                    .col(text_null(Bangumi::PosterLink))
                    .col(text_null(Bangumi::SavePath))
                    .col(unsigned(Bangumi::LastEp))
                    .col(jsonb_null(Bangumi::BangumiConfOverride))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bangumi_subscription_id")
                            .from(Bangumi::Table, Bangumi::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_bangumi_official_title")
                            .table(Bangumi::Table)
                            .col(Bangumi::OfficialTitle)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_bangumi_fansub")
                            .table(Bangumi::Table)
                            .col(Bangumi::Fansub)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_bangumi_display_name")
                            .table(Bangumi::Table)
                            .col(Bangumi::DisplayName)
                            .unique(),
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
                    .col(text(Episodes::OriginTitle))
                    .col(text(Episodes::OfficialTitle))
                    .col(text(Episodes::DisplayName))
                    .col(text_null(Episodes::NameZh))
                    .col(text_null(Episodes::NameJp))
                    .col(text_null(Episodes::NameEn))
                    .col(text_null(Episodes::SNameZh))
                    .col(text_null(Episodes::SNameJp))
                    .col(text_null(Episodes::SNameEn))
                    .col(integer(Episodes::BangumiId))
                    .col(integer(Episodes::ResourceId))
                    .col(text_null(Episodes::SavePath))
                    .col(string_null(Episodes::Resolution))
                    .col(integer(Episodes::Season))
                    .col(string_null(Episodes::SeasonRaw))
                    .col(string_null(Episodes::Fansub))
                    .col(text_null(Episodes::PosterLink))
                    .col(text_null(Episodes::HomePage))
                    .col(jsonb_null(Episodes::Subtitle))
                    .col(text_null(Episodes::Source))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episode_bangumi_id")
                            .from(Episodes::Table, Episodes::BangumiId)
                            .to(Bangumi::Table, Bangumi::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_episode_official_title")
                            .table(Episodes::Table)
                            .col(Episodes::OfficialTitle),
                    )
                    .index(
                        Index::create()
                            .name("idx_episode_fansub")
                            .table(Episodes::Table)
                            .col(Episodes::Fansub),
                    )
                    .index(
                        Index::create()
                            .name("idx_episode_display_name")
                            .table(Episodes::Table)
                            .col(Episodes::DisplayName),
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
