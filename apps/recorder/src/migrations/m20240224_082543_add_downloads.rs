use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::*;
use crate::models::prelude::{
    downloads::{DownloadMimeEnum, DownloadStatusEnum},
    DownloadMime, DownloadStatus,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_postgres_enum_for_active_enum!(
            manager,
            DownloadMimeEnum,
            DownloadMime::BitTorrent,
            DownloadMime::OctetStream
        )
        .await?;

        create_postgres_enum_for_active_enum!(
            manager,
            DownloadStatusEnum,
            DownloadStatus::Downloading,
            DownloadStatus::Paused,
            DownloadStatus::Pending,
            DownloadStatus::Completed,
            DownloadStatus::Failed,
            DownloadStatus::Deleted
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
                    .col(text(Downloads::Url))
                    .col(text_null(Downloads::Homepage))
                    .col(text_null(Downloads::SavePath))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_downloads_subscription_id")
                            .from(Downloads::Table, Downloads::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
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
                    .name("idx_downloads_url")
                    .table(Downloads::Table)
                    .col(Downloads::Url)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Episodes::Table)
                    .add_column_if_not_exists(integer_null(Episodes::DownloadId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_episodes_download_id")
                            .from_tbl(Episodes::Table)
                            .from_col(Episodes::DownloadId)
                            .to_tbl(Downloads::Table)
                            .to_col(Downloads::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::SetNull),
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
                    .drop_foreign_key(Alias::new("fk_episodes_download_id"))
                    .drop_column(Episodes::DownloadId)
                    .to_owned(),
            )
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
