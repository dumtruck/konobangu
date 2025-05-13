use async_graphql::{
    InputObject, InputValueResult, Scalar, ScalarType,
    dynamic::{ObjectAccessor, SchemaError, TypeRef},
};
use itertools::Itertools;
use maplit::btreeset;
use once_cell::sync::OnceCell;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait,
    prelude::Expr,
    sea_query::{self, IntoCondition, SimpleExpr, extension::postgres::PgExpr},
};
use seaography::{
    BuilderContext, FilterInfo, FilterOperation as SeaographqlFilterOperation, SeaResult,
};
use serde_json::Value;

use crate::errors::{RecorderError, RecorderResult};

pub static SUBSCRIBER_ID_FILTER_INFO: OnceCell<FilterInfo> = OnceCell::new();

pub fn init_custom_filter_info() {
    SUBSCRIBER_ID_FILTER_INFO.get_or_init(|| FilterInfo {
        type_name: String::from("SubscriberIdFilterInput"),
        base_type: TypeRef::INT.into(),
        supported_operations: btreeset! { SeaographqlFilterOperation::Equals },
    });
}

pub type FnFilterCondition =
    Box<dyn Fn(Condition, &ObjectAccessor) -> SeaResult<Condition> + Send + Sync>;

pub fn subscriber_id_condition_function<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterCondition
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(move |mut condition, filter| {
        let subscriber_id_filter_info = SUBSCRIBER_ID_FILTER_INFO.get().unwrap();
        let operations = &subscriber_id_filter_info.supported_operations;
        for operation in operations {
            match operation {
                SeaographqlFilterOperation::Equals => {
                    if let Some(value) = filter.get("eq") {
                        let value: i32 = value.i64()?.try_into()?;
                        let value = sea_orm::Value::Int(Some(value));
                        condition = condition.add(column.eq(value));
                    }
                }
                _ => unreachable!("unreachable filter operation for subscriber_id"),
            }
        }
        Ok(condition)
    })
}

#[derive(Clone, Debug, InputObject)]
pub struct StringFilterInput {
    pub eq: Option<Value>,
    pub ne: Option<Value>,
    pub gt: Option<Value>,
    pub gte: Option<Value>,
    pub lt: Option<Value>,
    pub lte: Option<Value>,
    pub in_: Option<Vec<Value>>,
    pub nin: Option<Vec<Value>>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
    pub contains: Option<Value>,
    pub starts_with: Option<Value>,
    pub ends_with: Option<Value>,
    pub like: Option<Value>,
    pub not_like: Option<Value>,
    pub between: Option<Value>,
    pub not_between: Option<Value>,
}

#[derive(Clone, Debug, InputObject)]
pub struct TextFilterInput {
    pub eq: Option<Value>,
    pub ne: Option<Value>,
    pub gt: Option<Value>,
    pub gte: Option<Value>,
    pub lt: Option<Value>,
    pub lte: Option<Value>,
    pub in_: Option<Vec<Value>>,
    pub nin: Option<Vec<Value>>,
    pub is_null: Option<bool>,
    pub between: Option<Value>,
    pub not_between: Option<Value>,
}

#[derive(Clone, Debug, InputObject)]
pub struct IntFilterInput {
    pub eq: Option<Value>,
    pub ne: Option<Value>,
    pub gt: Option<Value>,
    pub gte: Option<Value>,
    pub lt: Option<Value>,
    pub lte: Option<Value>,
    pub in_: Option<Vec<Value>>,
    pub nin: Option<Vec<Value>>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
    pub between: Option<Value>,
    pub not_between: Option<Value>,
}

#[derive(Clone, Debug, InputObject)]
pub struct FloatFilterInput {
    pub eq: Option<Value>,
    pub ne: Option<Value>,
    pub gt: Option<Value>,
    pub gte: Option<Value>,
    pub lt: Option<Value>,
    pub lte: Option<Value>,
    pub in_: Option<Vec<Value>>,
    pub nin: Option<Vec<Value>>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
    pub between: Option<Value>,
    pub not_between: Option<Value>,
}

#[derive(Clone, Debug, InputObject)]
pub struct BooleanFilterInput {
    pub eq: Option<Value>,
    pub ne: Option<Value>,
    pub gt: Option<Value>,
    pub gte: Option<Value>,
    pub lt: Option<Value>,
    pub lte: Option<Value>,
    pub in_: Option<Vec<Value>>,
    pub nin: Option<Vec<Value>>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
}

#[derive(Clone, Debug, InputObject)]
pub struct JsonArrayFilterInput {
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
    pub contains: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct JsonFilterInput(pub serde_json::Value);

#[Scalar(name = "JsonFilterInput")]
impl ScalarType for JsonFilterInput {
    fn parse(value: async_graphql::Value) -> InputValueResult<Self> {
        Ok(JsonFilterInput(value.into_json()?))
    }

    fn to_value(&self) -> async_graphql::Value {
        async_graphql::Value::from_json(self.0.clone()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum JsonFilterOperation {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanEquals,
    LessThan,
    LessThanEquals,
    IsIn,
    IsNotIn,
    IsNull,
    IsNotNull,
    Contains,
    StartsWith,
    EndsWith,
    Like,
    NotLike,
    Exists,
    NotExists,
    Between,
    NotBetween,
    And,
    Or,
}

impl JsonFilterOperation {
    pub fn is_filter_operation(property_key: &str) -> bool {
        property_key.starts_with("$")
    }

    pub fn parse_str(value: &str) -> Result<Option<Self>, async_graphql::dynamic::SchemaError> {
        match value {
            "$eq" => Ok(Some(JsonFilterOperation::Equals)),
            "$ne" => Ok(Some(JsonFilterOperation::NotEquals)),
            "$gt" => Ok(Some(JsonFilterOperation::GreaterThan)),
            "$gte" => Ok(Some(JsonFilterOperation::GreaterThanEquals)),
            "$lt" => Ok(Some(JsonFilterOperation::LessThan)),
            "$lte" => Ok(Some(JsonFilterOperation::LessThanEquals)),
            "$is_in" => Ok(Some(JsonFilterOperation::IsIn)),
            "$is_not_in" => Ok(Some(JsonFilterOperation::IsNotIn)),
            "$is_null" => Ok(Some(JsonFilterOperation::IsNull)),
            "$is_not_null" => Ok(Some(JsonFilterOperation::IsNotNull)),
            "$contains" => Ok(Some(JsonFilterOperation::Contains)),
            "$starts_with" => Ok(Some(JsonFilterOperation::StartsWith)),
            "$ends_with" => Ok(Some(JsonFilterOperation::EndsWith)),
            "$like" => Ok(Some(JsonFilterOperation::Like)),
            "$not_like" => Ok(Some(JsonFilterOperation::NotLike)),
            "$between" => Ok(Some(JsonFilterOperation::Between)),
            "$not_between" => Ok(Some(JsonFilterOperation::NotBetween)),
            "$and" => Ok(Some(JsonFilterOperation::And)),
            "$or" => Ok(Some(JsonFilterOperation::Or)),
            "$exists" => Ok(Some(JsonFilterOperation::Exists)),
            "$not_exists" => Ok(Some(JsonFilterOperation::NotExists)),
            s if Self::is_filter_operation(s) => Err(async_graphql::dynamic::SchemaError(format!(
                "Use reserved but not implemented filter operation: {value}"
            ))),
            _ => Ok(None),
        }
    }
}

impl AsRef<str> for JsonFilterOperation {
    fn as_ref(&self) -> &str {
        match self {
            JsonFilterOperation::Equals => "$eq",
            JsonFilterOperation::NotEquals => "$ne",
            JsonFilterOperation::GreaterThan => "$gt",
            JsonFilterOperation::GreaterThanEquals => "$gte",
            JsonFilterOperation::LessThan => "$lt",
            JsonFilterOperation::LessThanEquals => "$lte",
            JsonFilterOperation::IsIn => "$is_in",
            JsonFilterOperation::IsNotIn => "$is_not_in",
            JsonFilterOperation::IsNull => "$is_null",
            JsonFilterOperation::IsNotNull => "$is_not_null",
            JsonFilterOperation::Contains => "$contains",
            JsonFilterOperation::StartsWith => "$starts_with",
            JsonFilterOperation::EndsWith => "$ends_with",
            JsonFilterOperation::Like => "$like",
            JsonFilterOperation::NotLike => "$not_like",
            JsonFilterOperation::Between => "$between",
            JsonFilterOperation::NotBetween => "$not_between",
            JsonFilterOperation::And => "$and",
            JsonFilterOperation::Or => "$or",
            JsonFilterOperation::Exists => "$exists",
            JsonFilterOperation::NotExists => "$not_exists",
        }
    }
}

fn build_json_leaf_get_expr(
    expr: impl Into<SimpleExpr>,
    path: &[&str],
) -> RecorderResult<SimpleExpr> {
    if path.is_empty() {
        Err(async_graphql::dynamic::SchemaError(
            "JsonFilterInput path must be at least one level deep".to_string(),
        ))?
    }
    let mut expr = expr.into();
    for key in path {
        expr = expr.get_json_field(*key);
    }
    Ok(expr)
}

fn build_json_leaf_cast_expr(
    expr: impl Into<SimpleExpr>,
    path: &[&str],
) -> RecorderResult<SimpleExpr> {
    if path.is_empty() {
        Err(async_graphql::dynamic::SchemaError(
            "JsonFilterInput path must be at least one level deep".to_string(),
        ))?
    }
    let mut expr = expr.into();
    for key in path.iter().take(path.len() - 1) {
        expr = expr.get_json_field(*key);
    }
    expr = expr.cast_json_field(path[path.len() - 1]);
    Ok(expr)
}

fn build_json_path_expr(path: &[&str]) -> SimpleExpr {
    Expr::val(format!("$.{}", path.join("."))).into()
}

fn build_json_path_exists_expr(col_expr: impl Into<SimpleExpr>, path: &[&str]) -> SimpleExpr {
    Expr::cust_with_exprs(
        "JSON_EXISTS($1, $2)",
        [col_expr.into(), build_json_path_expr(path)],
    )
}

fn build_json_path_query_expr(col: impl Into<SimpleExpr>, path: &[&str]) -> SimpleExpr {
    Expr::cust_with_exprs("".to_string(), [col.into(), build_json_path_expr(path)])
}

fn build_json_value_is_in_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &[&str],
    values: Vec<Value>,
) -> RecorderResult<SimpleExpr> {
    let template = format!(
        "jsonb_path_query($1, $2) = ANY(ARRAY[{}]::jsonb[])",
        (0..values.len())
            .map(|i| format!("${}::jsonb", i + 3))
            .join(",")
    );
    let values = values
        .into_iter()
        .map(|v| serde_json::to_string(&v))
        .collect::<Result<Vec<_>, _>>()?;
    let mut exprs = vec![col_expr.into(), build_json_path_expr(path)];
    exprs.extend(values.into_iter().map(|v| Expr::val(v).into()));
    dbg!(&exprs);
    Ok(Expr::cust_with_exprs(template, exprs))
}

fn prepare_json_leaf_condition(
    col_expr: impl Into<SimpleExpr>,
    op: JsonFilterOperation,
    value: Value,
    path: &[&str],
) -> RecorderResult<Condition> {
    Ok(match (op, value) {
        (
            op @ (JsonFilterOperation::Exists | JsonFilterOperation::NotExists),
            Value::Bool(exists),
        ) => {
            let json_exists_expr = build_json_path_exists_expr(col_expr, path);
            if (op == JsonFilterOperation::Exists && exists)
                || (op == JsonFilterOperation::NotExists && !exists)
            {
                json_exists_expr.into_condition()
            } else {
                json_exists_expr.not().into_condition()
            }
        }
        (JsonFilterOperation::Exists | JsonFilterOperation::NotExists, _) => {
            Err(SchemaError(format!(
                "JsonFilterInput leaf can not be $exists or $not_exists with a non-boolean value"
            )))?
        }
        (JsonFilterOperation::And | JsonFilterOperation::Or, _) => {
            unreachable!("JsonFilterInput leaf can not be $and or $or with any value")
        }
        (JsonFilterOperation::Equals, value) => {
            let expr = build_json_leaf_get_expr(col_expr, path)?;
            expr.eq(value).into_condition()
        }
        (JsonFilterOperation::NotEquals, value) => {
            let expr = build_json_leaf_get_expr(col_expr, path)?;
            expr.ne(value).into_condition()
        }

        (
            JsonFilterOperation::GreaterThan
            | JsonFilterOperation::GreaterThanEquals
            | JsonFilterOperation::LessThan
            | JsonFilterOperation::LessThanEquals,
            Value::Array(_),
        ) => Err(SchemaError(format!(
            "JsonFilterInput leaf can not be {} with an array",
            op.as_ref()
        )))?,
        (_, _) => todo!(),
    })
}

// fn recursive_prepare_json_node_condition<'a, E>(
//     expr: &'a E,
//     node: Value,
//     mut path: Vec<&'a str>,
// ) -> RecorderResult<(Condition, Vec<&'a str>)>
// where
//     E: Into<SimpleExpr> + Clone,
// {
//     let object = node.as_object().ok_or(SchemaError(format!(
//         "Json filter input node must be an object"
//     )))?;

//     let mut conditions = Condition::all();

//     for (key, value) in object {
//         if let Some(operation) = JsonFilterOperation::parse_str(key)? {
//             match operation {
//                 JsonFilterOperation::And => {
//                     let mut condition = Condition::all();
//                     let filters = value.as_array().ok_or(SchemaError(format!(
//                         "$and operation must be an array of sub filters"
//                     )))?;

//                     for filter in filters {
//                         let result =
// recursive_prepare_json_node_condition(expr, filter, path)?;
// condition = condition.add(result.0);                         path = result.1;
//                     }

//                     conditions = conditions.add(condition);
//                 }
//                 JsonFilterOperation::Between => {
//                     let mut condition = Condition::any();
//                     let values = value
//                         .as_array()
//                         .and_then(|arr| if arr.len() == 2 { Some(arr) } else
// { None })                         .ok_or(SchemaError(format!(
//                             "$between operation must be an array of two
// values"                         )))?;

//                     let (lhs, rhs) = (values[0], values[1]);
//                     let (lcondition, lpath) =
//                         recursive_prepare_json_node_condition(expr, lhs,
// path)?;                     condition = condition.add(lcondition);
//                     let (rcondition, rpath) =
//                         recursive_prepare_json_node_condition(expr, rhs,
// lpath)?;                     condition = condition.add(rcondition);
//                     path = rpath;
//                     conditions = conditions.add(condition);
//                 }
//                 op => conditions.add(prepare_json_leaf_condition(expr, op,
// value, &path)?),             }
//         } else {
//             path.push(key as &'a str);
//             let result = recursive_prepare_json_node_condition(expr, node,
// path)?;             conditions = conditions.add(result.0);
//             path = result.1;
//             path.pop();
//         }
//     }

//     Ok((conditions, path))
// }

#[cfg(test)]
mod tests {
    use sea_orm::{
        DeriveIden,
        sea_query::{PostgresQueryBuilder, Query, Value, Values},
    };

    use super::*;

    #[derive(DeriveIden)]
    enum TestTable {
        Table,
        Job,
    }

    fn build_test_query_sql(where_expr: SimpleExpr) -> (String, Vec<Value>) {
        let (sql, Values(values)) = Query::select()
            .column(TestTable::Job)
            .and_where(where_expr)
            .from(TestTable::Table)
            .build(PostgresQueryBuilder);
        (sql, values)
    }

    #[test]
    fn test_build_json_path_exists_expr() {
        let (sql, params) = build_test_query_sql(build_json_path_exists_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &["a", "b", "c"],
        ));
        dbg!(&params);
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE JSON_EXISTS(\"test_table\".\"job\", $1)"
        );
        let expected_params = vec![Value::String(Some(Box::new("$.a.b.c".into())))];
        assert_eq!(params, expected_params);
    }

    #[test]
    fn test_build_json_path_query_expr() -> RecorderResult<()> {
        let (sql, params) = build_test_query_sql(build_json_value_is_in_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &["a", "b", "c"],
            vec![
                serde_json::json!(1),
                serde_json::json!("str"),
                serde_json::json!(true),
            ],
        )?);

        dbg!(&params);
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE jsonb_path_query(\"test_table\".\"job\", \
             $1) = ANY(ARRAY[$3::jsonb,$4::jsonb,$5::jsonb]::jsonb[])"
        );

        Ok(())
    }
}
