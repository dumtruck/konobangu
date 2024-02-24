use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::*;
use crate::models::prelude::{DownloadMime, DownloadStatus};
use crate::models::prelude::downloads::{DownloadMimeEnum, DownloadStatusEnum};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_postgres_enum_for_active_enum(
                DownloadMimeEnum,
                &[DownloadMime::OctetStream, DownloadMime::BitTorrent],
            )
            .await?;

        manager
            .create_postgres_enum_for_active_enum(
                DownloadStatusEnum,
                &[
                    DownloadStatus::Pending,
                    DownloadStatus::Downloading,
                    DownloadStatus::Completed,
                    DownloadStatus::Failed,
                    DownloadStatus::Deleted,
                    DownloadStatus::Paused,
                ],
            )
            .await?;

        manager
            .create_table(
                table_auto(Downloads::Table)
                    .col(pk_auto(Downloads::Id))
                    .col(string(Downloads::OriginalName))
                    .col(string(Downloads::DisplayName))
                    .col(integer(Downloads::SubscriptionId))
                    .col(enumeration(
                        Downloads::Status,
                        DownloadStatusEnum,
                        DownloadMime::iden_values(),
                    ))
                    .col(enumeration(
                        Downloads::Mime,
                        DownloadMimeEnum,
                        DownloadMime::iden_values(),
                    ))
                    .col(big_unsigned(Downloads::AllSize))
                    .col(big_unsigned(Downloads::CurrSize))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_download_subscription_id")
                            .from(Downloads::Table, Downloads::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt)
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Episodes::Table)
                    .add_column_if_not_exists(integer(Episodes::DownloadId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_episode_download_id")
                            .from_tbl(Episodes::Table)
                            .from_col(Episodes::DownloadId)
                            .to_tbl(Downloads::Table)
                            .to_col(Downloads::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Episodes::Table)
                    .drop_foreign_key(Alias::new("fk_episode_download_id"))
                    .drop_column(Episodes::DownloadId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Downloads::Table).to_owned())
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(DownloadMimeEnum)
            .await?;
        manager
            .drop_postgres_enum_for_active_enum(DownloadStatusEnum)
            .await?;

        Ok(())
    }
}
