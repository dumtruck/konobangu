use async_graphql::dynamic::SchemaError;
use itertools::Itertools;
use sea_orm::{
    Condition,
    sea_query::{
        ArrayType, Expr, ExprTrait, IntoCondition, SimpleExpr, Value as DbValue,
        extension::postgres::PgExpr,
    },
};
use serde_json::Value as JsonValue;

use crate::errors::RecorderResult;

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
    JsonbPathQuery,
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
            s if s.starts_with("$jsonb_path_query:") => {
                Ok(Some(JsonFilterOperation::JsonbPathQuery))
            }
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
            JsonFilterOperation::JsonbPathQuery => "$jsonb_path_query",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum JsonPathSegment {
    Str(String),
    Num(u64),
    JsonbPathQuery(String),
    Root,
}

impl TryFrom<&str> for JsonPathSegment {
    type Error = SchemaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(SchemaError("JsonPath segment can not be empty".to_string()))
        } else {
            Ok(JsonPathSegment::Str(value.to_string()))
        }
    }
}

impl TryFrom<String> for JsonPathSegment {
    type Error = SchemaError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(SchemaError("JsonPath segment can not be empty".to_string()))
        } else {
            Ok(JsonPathSegment::Str(value))
        }
    }
}

impl TryFrom<u64> for JsonPathSegment {
    type Error = SchemaError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(JsonPathSegment::Num(value))
    }
}

pub struct JsonPath(Vec<JsonPathSegment>);

impl Default for JsonPath {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonPath {
    pub fn new() -> Self {
        Self(vec![JsonPathSegment::Root])
    }

    pub fn push(&mut self, segment: impl Into<JsonPathSegment>) -> RecorderResult<()> {
        let s = segment.into();
        match &s {
            JsonPathSegment::Root => Err(SchemaError(
                "JsonPath can only contain one root segment".to_string(),
            ))?,
            JsonPathSegment::JsonbPathQuery(..) => {
                if !self
                    .0
                    .last()
                    .is_some_and(|s| matches!(s, JsonPathSegment::Root))
                {
                    Err(SchemaError(
                        "JsonPath jsonb_path_query must be the only non-root segment".to_string(),
                    ))?;
                }
                self.0.push(s);
            }
            _ => {
                if self
                    .0
                    .last()
                    .is_some_and(|s| !matches!(s, JsonPathSegment::JsonbPathQuery(..)))
                {
                    self.0.push(s);
                } else {
                    Err(SchemaError(
                        "JsonPath jsonb_path_query must be the only non-root segment".to_string(),
                    ))?;
                }
            }
        }
        Ok(())
    }

    fn pop(&mut self) -> Option<JsonPathSegment> {
        if self
            .0
            .last()
            .is_none_or(|s| matches!(s, JsonPathSegment::Root))
        {
            None
        } else {
            self.0.pop()
        }
    }

    fn join(&self) -> String {
        self.0
            .iter()
            .map(|s| match s {
                JsonPathSegment::Str(s) => {
                    let needs_brackets = s.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_');

                    if needs_brackets {
                        let escaped = s
                            .replace('\\', "\\\\")
                            .replace('\'', "\\'")
                            .replace('"', "\\\"");

                        format!("['{escaped}']")
                    } else {
                        format!(".{s}")
                    }
                }
                JsonPathSegment::Num(n) => format!("[{n}]"),
                JsonPathSegment::JsonbPathQuery(s) => s.into(),
                JsonPathSegment::Root => "$".into(),
            })
            .join("")
    }
}

fn build_json_path_expr(path: &JsonPath) -> SimpleExpr {
    Expr::val(path.join()).into()
}

fn build_json_path_exists_expr(col_expr: impl Into<SimpleExpr>, path: &JsonPath) -> SimpleExpr {
    Expr::cust_with_exprs(
        "JSON_EXISTS($1, $2)",
        [col_expr.into(), build_json_path_expr(path)],
    )
}

fn build_json_path_query_expr(col_expr: impl Into<SimpleExpr>, path: &JsonPath) -> SimpleExpr {
    Expr::cust_with_exprs(
        "jsonb_path_query($1, $2)",
        [col_expr.into(), build_json_path_expr(path)],
    )
}

fn build_json_value_is_in_values_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    values: Vec<JsonValue>,
) -> SimpleExpr {
    Expr::cust_with_exprs(
        "jsonb_path_query($1, $2) = ANY($3)",
        [
            col_expr.into(),
            build_json_path_expr(path),
            Expr::val(DbValue::Array(
                ArrayType::Json,
                Some(Box::new(
                    values
                        .into_iter()
                        .map(|v| DbValue::Json(Some(Box::new(v))))
                        .collect(),
                )),
            ))
            .into(),
        ],
    )
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

fn build_json_path_eq_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: JsonValue,
) -> SimpleExpr {
    build_json_path_query_expr(col_expr, path).eq(value)
}

fn build_json_path_ne_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: JsonValue,
) -> SimpleExpr {
    build_json_path_query_expr(col_expr, path).ne(value)
}

pub fn prepare_json_leaf_condition(
    col_expr: impl Into<SimpleExpr>,
    op: JsonFilterOperation,
    value: JsonValue,
    path: &JsonPath,
) -> RecorderResult<Condition> {
    Ok(match (op, value) {
        (
            op @ (JsonFilterOperation::Exists | JsonFilterOperation::NotExists),
            JsonValue::Bool(exists),
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
        (JsonFilterOperation::Exists | JsonFilterOperation::NotExists, _) => Err(SchemaError(
            "JsonFilterInput leaf can not be $exists or $not_exists with a non-boolean value"
                .to_string(),
        ))?,
        (JsonFilterOperation::And | JsonFilterOperation::Or, _) => {
            unreachable!("JsonFilterInput leaf can not be $and or $or with any value")
        }
        (JsonFilterOperation::Equals, value) => {
            build_json_path_eq_expr(col_expr, path, value).into_condition()
        }
        (JsonFilterOperation::NotEquals, value) => {
            build_json_path_ne_expr(col_expr, path, value).into_condition()
        }
        (
            op @ (JsonFilterOperation::IsIn | JsonFilterOperation::IsNotIn),
            JsonValue::Array(values),
        ) => {
            let expr = build_json_value_is_in_values_expr(col_expr, path, values.clone());
            if op == JsonFilterOperation::IsIn {
                expr.into_condition()
            } else {
                expr.not().into_condition()
            }
        }
        (JsonFilterOperation::IsIn | JsonFilterOperation::IsNotIn, _) => Err(SchemaError(
            "JsonFilterInput leaf can not be $is_in or $is_not_in with a non-array value"
                .to_string(),
        ))?,
        (
            op @ (JsonFilterOperation::IsNull | JsonFilterOperation::IsNotNull),
            JsonValue::Bool(is),
        ) => {
            let expr = build_json_path_query_expr(col_expr, path);
            if op == JsonFilterOperation::IsNull {
                if is {
                    expr.is_null().into_condition()
                } else {
                    expr.is_null().not().into_condition()
                }
            } else {
                if is {
                    expr.is_not_null().into_condition()
                } else {
                    expr.is_not_null().not().into_condition()
                }
            }
        }
        (
            JsonFilterOperation::GreaterThan
            | JsonFilterOperation::GreaterThanEquals
            | JsonFilterOperation::LessThan
            | JsonFilterOperation::LessThanEquals,
            JsonValue::Array(_),
        ) => Err(SchemaError(format!(
            "JsonFilterInput leaf can not be {} with an array",
            op.as_ref()
        )))?,
        _ => todo!(),
    })
}

pub fn recursive_prepare_json_node_condition<E>(
    expr: &E,
    mut node: JsonValue,
    mut path: JsonPath,
) -> RecorderResult<(Condition, JsonPath)>
where
    E: Into<SimpleExpr> + Clone,
{
    let object = {
        let a = node.as_object_mut().ok_or(SchemaError(
            "Json filter input node must be an object".to_string(),
        ))?;
        let mut b = serde_json::Map::new();
        std::mem::swap(a, &mut b);
        b
    };

    let mut conditions = Condition::all();

    for (key, mut value) in object {
        if let Some(operation) = JsonFilterOperation::parse_str(&key)? {
            match operation {
                JsonFilterOperation::And => {
                    let mut condition = Condition::all();
                    let filters = {
                        let a = value.as_array_mut().ok_or(SchemaError(
                            "$and operation must be an array of sub filters".to_string(),
                        ))?;
                        let mut b = vec![];
                        std::mem::swap(a, &mut b);
                        b
                    };

                    for filter in filters {
                        let result = recursive_prepare_json_node_condition(expr, filter, path)?;
                        condition = condition.add(result.0);
                        path = result.1;
                    }

                    conditions = conditions.add(condition);
                }
                JsonFilterOperation::Or => {
                    let mut condition = Condition::any();
                    let mut values = {
                        let a = value
                            .as_array_mut()
                            .and_then(|arr| if arr.len() == 2 { Some(arr) } else { None })
                            .ok_or(SchemaError(
                                "$between operation must be an array of two values".to_string(),
                            ))?;
                        let mut b = vec![];
                        std::mem::swap(a, &mut b);
                        b
                    };

                    let (lhs, rhs) = (values.pop().unwrap(), values.pop().unwrap());
                    let (lcondition, lpath) =
                        recursive_prepare_json_node_condition(expr, lhs, path)?;
                    condition = condition.add(lcondition);
                    let (rcondition, rpath) =
                        recursive_prepare_json_node_condition(expr, rhs, lpath)?;
                    condition = condition.add(rcondition);
                    path = rpath;
                    conditions = conditions.add(condition);
                }
                JsonFilterOperation::JsonbPathQuery => {
                    path.push(JsonPathSegment::JsonbPathQuery(
                        key.split_at(16).1.to_string(),
                    ))?;
                    let (condition, rpath) =
                        recursive_prepare_json_node_condition(expr, value, path)?;
                    conditions = conditions.add(condition);
                    path = rpath;
                    path.pop();
                }
                op => {
                    let condition = prepare_json_leaf_condition(expr.clone(), op, value, &path)?;
                    conditions = conditions.add(condition);
                }
            }
        } else {
            path.push(JsonPathSegment::Str(key))?;
            let result = recursive_prepare_json_node_condition(expr, value, path)?;
            conditions = conditions.add(result.0);
            path = result.1;
            path.pop();
        }
    }

    Ok((conditions, path))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use sea_orm::{
        DeriveIden,
        sea_query::{PostgresQueryBuilder, Query, Value, Values},
    };
    use serde_json::json;

    use super::*;
    use crate::errors::RecorderResult;

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

    fn build_test_json_path(path: &[&str]) -> JsonPath {
        let mut p = JsonPath::new();
        for s in path {
            p.push(JsonPathSegment::Str(s.to_string())).unwrap();
        }
        p
    }

    #[test]
    fn test_build_json_path_exists_expr() {
        let (sql, params) = build_test_query_sql(build_json_path_exists_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE JSON_EXISTS(\"test_table\".\"job\", $1)"
        );
        let expected_params = vec![Value::String(Some(Box::new("$.a.b.c".into())))];
        assert_eq!(params, expected_params);
    }

    #[test]
    fn test_build_json_path_query_expr() -> RecorderResult<()> {
        let (sql, params) = build_test_query_sql(build_json_value_is_in_values_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            vec![json!(1), json!("str"), json!(true)],
        ));

        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE jsonb_path_query(\"test_table\".\"job\", \
             $1) = ANY($2)"
        );
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], DbValue::String(Some(Box::new("$.a.b.c".into()))));
        assert_matches!(params[1], DbValue::Array(..));

        Ok(())
    }

    #[test]
    fn test_build_json_path_eq_expr() -> RecorderResult<()> {
        let (sql, params) = build_test_query_sql(build_json_path_eq_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            json!("str"),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE (jsonb_path_query(\"test_table\".\"job\", \
             $1)) = $2"
        );
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], DbValue::String(Some(Box::new("$.a.b.c".into()))));
        assert_eq!(params[1], DbValue::Json(Some(Box::new(json!("str")))));

        Ok(())
    }
}
