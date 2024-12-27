use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    migrations::defs::{CustomSchemaManagerExt, Downloaders, GeneralIds, Subscribers},
    models::{downloaders::DownloaderCategoryEnum, prelude::DownloaderCategory},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_postgres_enum_for_active_enum!(
            manager,
            DownloaderCategoryEnum,
            DownloaderCategory::QBittorrent
        )
        .await?;

        manager
            .create_table(
                table_auto(Downloaders::Table)
                    .col(pk_auto(Downloaders::Id))
                    .col(text(Downloaders::Endpoint))
                    .col(string_null(Downloaders::Username))
                    .col(string_null(Downloaders::Password))
                    .col(enumeration(
                        Downloaders::Category,
                        DownloaderCategoryEnum,
                        DownloaderCategory::iden_values(),
                    ))
                    .col(text(Downloaders::SavePath))
                    .col(integer(Downloaders::SubscriberId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_downloader_subscriber_id")
                            .from_tbl(Downloaders::Table)
                            .from_col(Downloaders::SubscriberId)
                            .to_tbl(Subscribers::Table)
                            .to_col(Subscribers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(
                Downloaders::Table,
                GeneralIds::UpdatedAt,
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Subscribers::Table)
                    .add_column_if_not_exists(integer_null(Subscribers::DownloaderId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_subscribers_downloader_id")
                            .from_tbl(Subscribers::Table)
                            .from_col(Subscribers::DownloaderId)
                            .to_tbl(Downloaders::Table)
                            .to_col(Downloaders::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Restrict),
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
                    .table(Subscribers::Table)
                    .drop_foreign_key(Alias::new("fk_subscribers_downloader_id"))
                    .drop_column(Subscribers::DownloaderId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Downloaders::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Downloaders::Table).to_owned())
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(DownloaderCategoryEnum)
            .await?;

        Ok(())
    }
}
