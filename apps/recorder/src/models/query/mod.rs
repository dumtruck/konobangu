use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, Insert, IntoActiveModel,
    Iterable, QueryResult, QueryTrait, SelectModel, SelectorRaw, Value,
    prelude::Expr,
    sea_query::{Alias, IntoColumnRef, IntoTableRef, Query, SelectStatement},
};

pub fn filter_values_in<
    I: IntoIterator<Item = T>,
    T: Into<Value>,
    R: IntoTableRef,
    C: IntoColumnRef + Copy,
>(
    tbl_ref: R,
    col_ref: C,
    values: I,
) -> SelectStatement {
    Query::select()
        .expr(Expr::col(("t", "column1")))
        .from_values(values, "t")
        .left_join(tbl_ref, Expr::col(("t", "column1")).equals(col_ref))
        .and_where(Expr::col(col_ref).is_not_null())
        .to_owned()
}

#[async_trait]
pub trait InsertManyReturningExt<A>: Sized
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait,
{
    fn exec_with_returning_models<C>(
        self,
        db: &C,
    ) -> SelectorRaw<SelectModel<<A::Entity as EntityTrait>::Model>>
    where
        C: ConnectionTrait;

    async fn exec_with_returning_columns<C, I>(
        self,
        db: &C,
        columns: I,
    ) -> Result<Vec<QueryResult>, DbErr>
    where
        C: ConnectionTrait,
        I: IntoIterator<Item = <A::Entity as EntityTrait>::Column> + Send;
}

#[async_trait]
impl<A> InsertManyReturningExt<A> for Insert<A>
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait + Send,
{
    fn exec_with_returning_models<C>(
        self,
        db: &C,
    ) -> SelectorRaw<SelectModel<<A::Entity as EntityTrait>::Model>>
    where
        C: ConnectionTrait,
    {
        let mut insert_statement = self.into_query();
        let db_backend = db.get_database_backend();
        let returning = Query::returning().exprs(
            <A::Entity as EntityTrait>::Column::iter()
                .map(|c| c.select_as(c.into_returning_expr(db_backend))),
        );
        insert_statement.returning(returning);
        let insert_statement = db_backend.build(&insert_statement);
        SelectorRaw::<SelectModel<<A::Entity as EntityTrait>::Model>>::from_statement(
            insert_statement,
        )
    }

    async fn exec_with_returning_columns<C, I>(
        self,
        db: &C,
        columns: I,
    ) -> Result<Vec<QueryResult>, DbErr>
    where
        C: ConnectionTrait,
        I: IntoIterator<Item = <A::Entity as EntityTrait>::Column> + Send,
    {
        let mut insert_statement = self.into_query();
        let db_backend = db.get_database_backend();
        let returning = Query::returning().exprs(
            columns
                .into_iter()
                .map(|c| c.select_as(c.into_returning_expr(db_backend))),
        );
        insert_statement.returning(returning);

        let statement = db_backend.build(&insert_statement);

        db.query_all(statement).await
    }
}
