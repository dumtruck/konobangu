use sea_orm::{
    prelude::Expr,
    sea_query::{Alias, IntoColumnRef, IntoTableRef, Query, SelectStatement},
    Value,
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
        .expr(Expr::col((Alias::new("t"), Alias::new("column1"))))
        .from_values(values, Alias::new("t"))
        .left_join(
            tbl_ref,
            Expr::col((Alias::new("t"), Alias::new("column1"))).equals(col_ref),
        )
        .and_where(Expr::col(col_ref).is_not_null())
        .to_owned()
}
