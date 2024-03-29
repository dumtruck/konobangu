use sea_orm::{
    sea_query::{Expr, InsertStatement, IntoIden, Query, SimpleExpr},
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityName, EntityTrait,
    FromQueryResult, Iterable, SelectModel, SelectorRaw, TryGetable,
};

use crate::migrations::{defs::GeneralIds, ColumnRef};

#[derive(FromQueryResult)]
pub(crate) struct OnlyIdsModel<Id>
where
    Id: TryGetable,
{
    pub id: Id,
}

pub(crate) async fn insert_many_with_returning_columns<M, D, V, T, F>(
    db: &D,
    insert_values: impl IntoIterator<Item = V>,
    returning_columns: impl IntoIterator<Item = T>,
    extra_config: F,
) -> eyre::Result<Vec<M>>
where
    D: ConnectionTrait,
    V: ActiveModelTrait,
    T: Into<SimpleExpr>,
    F: FnOnce(&mut InsertStatement),
    M: FromQueryResult,
{
    let db_backend = db.get_database_backend();
    assert!(
        db_backend.support_returning(),
        "db backend must support returning!"
    );
    let ent = V::Entity::default();
    let mut insert = Query::insert();
    let insert_statement = insert
        .into_table(ent.table_ref())
        .returning(Query::returning().exprs(returning_columns));

    {
        extra_config(insert_statement);
    }

    for new_item in insert_values {
        let mut columns = vec![];
        let mut values = vec![];
        for c in <V::Entity as EntityTrait>::Column::iter() {
            let av = new_item.get(c);
            match av {
                ActiveValue::Set(value) => {
                    values.push(c.save_as(Expr::val(value)));
                    columns.push(c);
                }
                ActiveValue::Unchanged(value) => {
                    values.push(c.save_as(Expr::val(value)));
                    columns.push(c);
                }
                _ => {}
            }
        }
        insert_statement.columns(columns);
        insert_statement.values(values)?;
    }

    let result = SelectorRaw::<SelectModel<M>>::from_statement(db_backend.build(insert_statement))
        .all(db)
        .await?;

    Ok(result)
}

pub(crate) async fn insert_many_with_returning_all<D, V, F>(
    db: &D,
    insert_values: impl IntoIterator<Item = V>,
    extra_config: F,
) -> eyre::Result<Vec<<V::Entity as EntityTrait>::Model>>
where
    D: ConnectionTrait,
    V: ActiveModelTrait,
    F: FnOnce(&mut InsertStatement),
{
    let result: Vec<<V::Entity as EntityTrait>::Model> = insert_many_with_returning_columns(
        db,
        insert_values,
        <V::Entity as EntityTrait>::Column::iter().map(|c| c.select_as(Expr::col(c))),
        extra_config,
    )
    .await?;

    Ok(result)
}

pub(crate) async fn insert_many_with_returning_id<D, V, F, I>(
    db: &D,
    insert_values: impl IntoIterator<Item = V>,
    extra_config: F,
) -> eyre::Result<Vec<OnlyIdsModel<I>>>
where
    D: ConnectionTrait,
    V: ActiveModelTrait,
    F: FnOnce(&mut InsertStatement),
    I: TryGetable,
{
    let result: Vec<OnlyIdsModel<I>> = insert_many_with_returning_columns(
        db,
        insert_values,
        [Expr::col(ColumnRef::Column(GeneralIds::Id.into_iden()))],
        extra_config,
    )
    .await?;

    Ok(result)
}
