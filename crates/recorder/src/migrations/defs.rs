use std::{collections::HashSet};
use std::fmt::Display;

use sea_orm::{DeriveIden, Statement};
use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::IntoTypeRef;

use crate::migrations::extension::postgres::Type;

#[derive(DeriveIden)]
pub enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Subscribers {
    Table,
    Id,
    Pid,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum Subscriptions {
    Table,
    Id,
    SubscriberId,
    DisplayName,
    Category,
    SourceUrl,
    Aggregate,
    Enabled,
}

#[derive(DeriveIden)]
pub enum Bangumi {
    Table,
    Id,
    DisplayName,
    SubscriptionId,
}

#[derive(DeriveIden)]
pub enum Episodes {
    Table,
    Id,
    DisplayName,
    BangumiId,
    OutputName,
    DownloadId,
}

#[derive(DeriveIden)]
pub enum Downloads {
    Table,
    Id,
    SubscriptionId,
    OriginalName,
    DisplayName,
    Status,
    CurrSize,
    AllSize,
    Mime,
    Url,
}

#[async_trait::async_trait]
pub trait CustomSchemaManagerExt {
    async fn create_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr>;
    async fn create_postgres_auto_update_ts_fn_for_col<C: IntoIden + 'static + Send>(
        &self,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        self.create_postgres_auto_update_ts_fn(&column_ident.to_string())
            .await?;
        Ok(())
    }

    async fn create_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr>;

    async fn create_postgres_auto_update_ts_trigger_for_col<
        T: IntoIden + 'static + Send,
        C: IntoIden + 'static + Send,
    >(
        &self,
        tab: T,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        let table_ident = tab.into_iden();
        self.create_postgres_auto_update_ts_trigger(
            &table_ident.to_string(),
            &column_ident.to_string(),
        )
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr>;

    async fn drop_postgres_auto_update_ts_fn_for_col<C: IntoIden + Send>(
        &self,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        self.drop_postgres_auto_update_ts_fn(&column_ident.to_string())
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr>;

    async fn drop_postgres_auto_update_ts_trigger_for_col<
        T: IntoIden + 'static + Send,
        C: IntoIden + 'static + Send,
    >(
        &self,
        tab: T,
        col: C,
    ) -> Result<(), DbErr> {
        let column_ident = col.into_iden();
        let table_ident = tab.into_iden();
        self.drop_postgres_auto_update_ts_trigger(
            &table_ident.to_string(),
            &column_ident.to_string(),
        )
            .await?;
        Ok(())
    }

    async fn create_postgres_enum_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        T: Display + Send,
        I: IntoIterator<Item=T> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr>;

    async fn add_postgres_enum_values_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        T: Display + Send,
        I: IntoIterator<Item=T> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr>;

    async fn drop_postgres_enum_for_active_enum<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<(), DbErr>;

    async fn if_postgres_enum_exists<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<bool, DbErr>;

    async fn get_postgres_enum_values<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<HashSet<String>, DbErr>;
}

#[async_trait::async_trait]
impl<'c> CustomSchemaManagerExt for SchemaManager<'c> {
    async fn create_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr> {
        let sql = format!(
            "CREATE OR REPLACE FUNCTION update_{col_name}_column() RETURNS TRIGGER AS $$ BEGIN \
             NEW.{col_name}  = current_timestamp; RETURN NEW; END; $$ language 'plpgsql';"
        );

        self.get_connection()
            .execute(Statement::from_string(self.get_database_backend(), sql))
            .await?;

        Ok(())
    }

    async fn create_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr> {
        let sql = format!(
            "CREATE OR REPLACE TRIGGER update_{tab_name}_{col_name}_column_trigger BEFORE UPDATE \
             ON {tab_name} FOR EACH ROW EXECUTE PROCEDURE update_{col_name}_column();"
        );
        self.get_connection()
            .execute(Statement::from_string(self.get_database_backend(), sql))
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_fn(&self, col_name: &str) -> Result<(), DbErr> {
        let sql = format!("DROP FUNCTION IF EXISTS update_{col_name}_column();");
        self.get_connection()
            .execute(Statement::from_string(self.get_database_backend(), sql))
            .await?;
        Ok(())
    }

    async fn drop_postgres_auto_update_ts_trigger(
        &self,
        tab_name: &str,
        col_name: &str,
    ) -> Result<(), DbErr> {
        let sql = format!(
            "DROP TRIGGER IF EXISTS update_{tab_name}_{col_name}_column_trigger ON {tab_name};"
        );
        self.get_connection()
            .execute(Statement::from_string(self.get_database_backend(), sql))
            .await?;
        Ok(())
    }

    async fn create_postgres_enum_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        T: Display + Send,
        I: IntoIterator<Item=T> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr> {
        let existed = self.if_postgres_enum_exists(enum_name.clone()).await?;
        if !existed {
            let idents = values
                .into_iter()
                .map(|v| Alias::new(v.to_string()))
                .collect::<Vec<_>>();
            self.create_type(
                Type::create()
                    .as_enum(enum_name)
                    .values(idents)
                    .to_owned(),
            )
                .await?;
        } else {
            self.add_postgres_enum_values_for_active_enum(enum_name, values)
                .await?;
        }
        Ok(())
    }

    async fn add_postgres_enum_values_for_active_enum<
        E: IntoTypeRef + IntoIden + Send + Clone,
        T: Display + Send,
        I: IntoIterator<Item=T> + Send,
    >(
        &self,
        enum_name: E,
        values: I,
    ) -> Result<(), DbErr> {
        let exists_values = self.get_postgres_enum_values(enum_name.clone()).await?;
        let to_add_values = values
            .into_iter()
            .filter(|v| !exists_values.contains(&v.to_string()))
            .collect::<Vec<_>>();

        if to_add_values.is_empty() {
            return Ok(());
        }

        let mut type_alter = Type::alter().name(enum_name);

        for v in to_add_values {
            type_alter = type_alter.add_value(Alias::new(v.to_string()));
        }

        self.alter_type(type_alter.to_owned()).await?;
        Ok(())
    }

    async fn drop_postgres_enum_for_active_enum<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<(), DbErr> {
        self.drop_type(Type::drop().name(enum_name).to_owned())
            .await?;
        Ok(())
    }

    async fn if_postgres_enum_exists<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<bool, DbErr> {
        let enum_name: String = enum_name.into_iden().to_string();
        let sql = format!("SELECT 1 FROM pg_type WHERE typname = '{enum_name}'");
        let result = self
            .get_connection()
            .query_one(Statement::from_string(self.get_database_backend(), sql))
            .await?;
        Ok(result.is_some())
    }

    async fn get_postgres_enum_values<E: IntoTypeRef + IntoIden + Send + Clone>(
        &self,
        enum_name: E,
    ) -> Result<HashSet<String>, DbErr> {
        let enum_name: String = enum_name.into_iden().to_string();
        let sql = format!(
            "SELECT pg_enum.enumlabel AS enumlabel FROM pg_type JOIN pg_enum ON pg_enum.enumtypid \
             = pg_type.oid WHERE pg_type.typname = '{enum_name}';"
        );

        let results = self
            .get_connection()
            .query_all(Statement::from_string(self.get_database_backend(), sql))
            .await?;

        let mut items = HashSet::new();
        for r in results {
            items.insert(r.try_get::<String>("", "enumlabel")?);
        }

        Ok(items)
    }
}
