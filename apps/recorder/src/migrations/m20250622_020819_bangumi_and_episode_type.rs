use async_trait::async_trait;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    migrations::defs::{Bangumi, CustomSchemaManagerExt, Episodes},
    models::{
        bangumi::{BangumiType, BangumiTypeEnum},
        episodes::{EpisodeType, EpisodeTypeEnum},
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_postgres_enum_for_active_enum!(manager, EpisodeTypeEnum, EpisodeType::Mikan).await?;

        {
            create_postgres_enum_for_active_enum!(manager, BangumiTypeEnum, BangumiType::Mikan)
                .await?;
            manager
                .alter_table(
                    Table::alter()
                        .table(Bangumi::Table)
                        .add_column_if_not_exists(enumeration_null(
                            Bangumi::BangumiType,
                            BangumiTypeEnum,
                            BangumiType::iden_values(),
                        ))
                        .drop_column(Bangumi::SavePath)
                        .to_owned(),
                )
                .await?;

            manager
                .exec_stmt(
                    UpdateStatement::new()
                        .table(Bangumi::Table)
                        .value(
                            Bangumi::BangumiType,
                            BangumiType::Mikan.as_enum(BangumiTypeEnum),
                        )
                        .and_where(Expr::col(Bangumi::BangumiType).is_null())
                        .and_where(Expr::col(Bangumi::MikanBangumiId).is_not_null())
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Bangumi::Table)
                        .modify_column(enumeration(
                            Bangumi::BangumiType,
                            BangumiTypeEnum,
                            BangumiType::iden_values(),
                        ))
                        .to_owned(),
                )
                .await?;
        }

        {
            create_postgres_enum_for_active_enum!(manager, EpisodeTypeEnum, EpisodeType::Mikan)
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Episodes::Table)
                        .add_column_if_not_exists(enumeration_null(
                            Episodes::EpisodeType,
                            EpisodeTypeEnum,
                            EpisodeType::enum_type_name(),
                        ))
                        .add_column_if_not_exists(text_null(Episodes::EnclosureMagnetLink))
                        .add_column_if_not_exists(text_null(Episodes::EnclosureTorrentLink))
                        .add_column_if_not_exists(timestamp_with_time_zone_null(
                            Episodes::EnclosurePubDate,
                        ))
                        .add_column_if_not_exists(big_integer_null(
                            Episodes::EnclosureContentLength,
                        ))
                        .drop_column(Episodes::SavePath)
                        .to_owned(),
                )
                .await?;

            manager
                .exec_stmt(
                    UpdateStatement::new()
                        .table(Episodes::Table)
                        .value(
                            Episodes::EpisodeType,
                            EpisodeType::Mikan.as_enum(EpisodeTypeEnum),
                        )
                        .and_where(Expr::col(Episodes::EpisodeType).is_null())
                        .and_where(Expr::col(Episodes::MikanEpisodeId).is_not_null())
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Episodes::Table)
                        .modify_column(enumeration(
                            Episodes::EpisodeType,
                            EpisodeTypeEnum,
                            EpisodeType::enum_type_name(),
                        ))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_postgres_enum_for_active_enum(BangumiTypeEnum)
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(EpisodeTypeEnum)
            .await?;

        Ok(())
    }
}
