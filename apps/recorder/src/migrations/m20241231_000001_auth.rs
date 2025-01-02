use sea_orm_migration::{prelude::*, schema::*};

use super::defs::Auth;
use crate::{
    migrations::defs::{CustomSchemaManagerExt, GeneralIds, Subscribers},
    models::{
        auth::{AuthType, AuthTypeEnum},
        subscribers::SEED_SUBSCRIBER,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_postgres_enum_for_active_enum!(
            manager,
            AuthTypeEnum,
            AuthType::Basic,
            AuthType::Oidc
        )
        .await?;

        manager
            .create_table(
                table_auto(Auth::Table)
                    .col(pk_auto(Auth::Id))
                    .col(text(Auth::Pid))
                    .col(enumeration(
                        Auth::AuthType,
                        AuthTypeEnum,
                        AuthType::iden_values(),
                    ))
                    .col(string_null(Auth::AvatarUrl))
                    .col(integer(Auth::SubscriberId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_auth_subscriber_id")
                            .from_tbl(Auth::Table)
                            .from_col(Auth::SubscriberId)
                            .to_tbl(Subscribers::Table)
                            .to_col(Subscribers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_auth_pid_auth_type")
                    .unique()
                    .table(Auth::Table)
                    .col(Auth::Pid)
                    .col(Auth::AuthType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_postgres_auto_update_ts_trigger_for_col(Auth::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Auth::Table)
                    .columns([Auth::Pid, Auth::AuthType, Auth::SubscriberId])
                    .values_panic([
                        SEED_SUBSCRIBER.into(),
                        SimpleExpr::from(AuthType::Basic).as_enum(AuthTypeEnum),
                        1.into(),
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_postgres_auto_update_ts_trigger_for_col(Auth::Table, GeneralIds::UpdatedAt)
            .await?;

        manager
            .drop_table(Table::drop().table(Auth::Table).to_owned())
            .await?;

        manager
            .drop_postgres_enum_for_active_enum(AuthTypeEnum)
            .await?;

        Ok(())
    }
}
