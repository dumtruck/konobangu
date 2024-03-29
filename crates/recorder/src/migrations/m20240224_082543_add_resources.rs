use loco_rs::schema::table_auto;
use sea_orm_migration::{prelude::*, schema::*};

use super::defs::*;
use crate::models::resources::{
    DownloadStatus, DownloadStatusEnum, ResourceCategory, ResourceCategoryEnum,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_postgres_enum_for_active_enum(
                ResourceCategoryEnum,
                [
                    ResourceCategory::BitTorrent,
                    ResourceCategory::OctetStream,
                    ResourceCategory::Poster,
                ],
            )
            .await?;

        manager
            .create_postgres_enum_for_active_enum(
                DownloadStatusEnum,
                [
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
                table_auto(Resources::Table)
                    .col(pk_auto(Resources::Id))
                    .col(text(Resources::OriginTitle))
                    .col(text(Resources::DisplayName))
                    .col(integer(Resources::SubscriptionId))
                    .col(enumeration(
                        Resources::Status,
                        DownloadStatusEnum,
                        ResourceCategory::iden_values(),
                    ))
                    .col(enumeration(
                        Resources::Category,
                        ResourceCategoryEnum,
                        ResourceCategory::iden_values(),
                    ))
                    .col(big_integer_null(Resources::AllSize))
                    .col(big_integer_null(Resources::CurrSize))
                    .col(text(Resources::Url))
                    .col(text_null(Resources::Homepage))
                    .col(text_null(Resources::SavePath))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_download_subscription_id")
                            .from(Resources::Table, Resources::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_update(ForeignKeyAction::Restrict)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        manager
                            .build_convention_index(Resources::Table, [Resources::Url])
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        futures::try_join!(
            manager.create_convention_index(Resources::Table, [Resources::Homepage]),
        )?;

        manager
            .create_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt)
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Episodes::Table)
                    .add_column_if_not_exists(integer_null(Episodes::ResourceId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_episode_resource_id")
                            .from_tbl(Episodes::Table)
                            .from_col(Episodes::ResourceId)
                            .to_tbl(Resources::Table)
                            .to_col(Resources::Id)
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
                    .drop_foreign_key(Alias::new("fk_episode_resource_id"))
                    .drop_column(Episodes::ResourceId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_postgres_auto_update_ts_fn_for_col(GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Resources::Table).to_owned())
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(ResourceCategoryEnum)
            .await?;
        manager
            .drop_postgres_enum_for_active_enum(DownloadStatusEnum)
            .await?;

        Ok(())
    }
}
