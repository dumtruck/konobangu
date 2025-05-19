use async_graphql::{
    Error as GraphqlError, InputValueResult, Scalar, ScalarType, dynamic::SchemaError, to_value,
};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use sea_orm::{
    Condition, EntityTrait,
    sea_query::{ArrayType, Expr, ExprTrait, IntoLikeExpr, SimpleExpr, Value as DbValue},
};
use seaography::{BuilderContext, FilterInfo, SeaographyError};
use serde_json::Value as JsonValue;

use super::subscriber::FnFilterCondition;
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
    Any,
    Not,
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
            "$any" => Ok(Some(JsonFilterOperation::Any)),
            "$not" => Ok(Some(JsonFilterOperation::Not)),
            s if s.starts_with("$query:") && s.len() >= 7 => {
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
            JsonFilterOperation::JsonbPathQuery => "$query",
            JsonFilterOperation::Any => "$any",
            JsonFilterOperation::Not => "$not",
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

fn json_path_expr(path: &JsonPath) -> SimpleExpr {
    Expr::val(path.join()).into()
}

fn json_path_exists_expr(col_expr: impl Into<SimpleExpr>, path: &JsonPath) -> SimpleExpr {
    Expr::cust_with_exprs(
        "jsonb_path_exists($1, $2)",
        [col_expr.into(), json_path_expr(path)],
    )
}

fn json_path_query_first_expr(col_expr: impl Into<SimpleExpr>, path: &JsonPath) -> SimpleExpr {
    Expr::cust_with_exprs(
        "jsonb_path_query_first($1, $2)",
        [col_expr.into(), json_path_expr(path)],
    )
}

fn json_path_query_first_auto_cast_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: &JsonValue,
) -> RecorderResult<SimpleExpr> {
    let cast_target = match value {
        JsonValue::Number(..) => "numeric",
        JsonValue::Bool(..) => "boolean",
        JsonValue::String(..) => "text",
        _ => {
            return Err(SchemaError(
                "JsonFilterInput leaf can not be only be casted to numeric, boolean or text"
                    .to_string(),
            ))?;
        }
    };
    Ok(json_path_query_first_expr(col_expr, path).cast_as(cast_target))
}

fn json_path_is_in_values_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    values: Vec<JsonValue>,
) -> SimpleExpr {
    Expr::cust_with_exprs(
        "$1 = ANY($2)",
        [
            json_path_query_first_expr(col_expr, path),
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

fn json_path_eq_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: JsonValue,
) -> SimpleExpr {
    json_path_query_first_expr(col_expr, path).eq(value)
}

fn json_path_ne_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: JsonValue,
) -> SimpleExpr {
    json_path_query_first_expr(col_expr, path).ne(value)
}

fn json_path_type_assert_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    typestr: &str,
) -> SimpleExpr {
    Expr::cust_with_exprs(
        format!("jsonb_path_exists($1, $2 || ' ? (@.type() = \"{typestr}\")')"),
        [col_expr.into(), json_path_expr(path)],
    )
}

fn json_path_is_null_expr(col_expr: impl Into<SimpleExpr>, path: &JsonPath) -> SimpleExpr {
    Expr::cust_with_exprs(
        "jsonb_path_exists($1, $2 || ' ? (@ == null)')",
        [col_expr.into(), json_path_expr(path)],
    )
}

fn json_path_is_not_null_expr(col_expr: impl Into<SimpleExpr>, path: &JsonPath) -> SimpleExpr {
    Expr::cust_with_exprs(
        "jsonb_path_exists($1, $2 || ' ? (@ != null)')",
        [col_expr.into(), json_path_expr(path)],
    )
}

fn convert_json_number_to_db_decimal(json_number: serde_json::Number) -> RecorderResult<Decimal> {
    if let Some(f) = json_number.as_f64() {
        let decimal = Decimal::from_f64(f).ok_or_else(|| {
            SchemaError("JsonFilterInput leaf value failed to convert to decimal".to_string())
        })?;
        Ok(decimal)
    } else if let Some(i) = json_number.as_i64() {
        Ok(Decimal::from(i))
    } else if let Some(u) = json_number.as_u64() {
        Ok(Decimal::from(u))
    } else {
        Err(
            SchemaError("JsonFilterInput leaf value failed to convert to a number".to_string())
                .into(),
        )
    }
}

fn json_path_like_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: impl IntoLikeExpr,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "string");
    left.and(
        json_path_query_first_expr(col_expr, path)
            .cast_as("text")
            .like(value),
    )
}

fn json_path_not_like_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: impl IntoLikeExpr,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "string");
    left.and(
        json_path_query_first_expr(col_expr, path)
            .cast_as("text")
            .not_like(value),
    )
}

fn json_path_starts_with_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: String,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let type_assert_expr = json_path_type_assert_expr(col_expr.clone(), path, "string");
    let get_value_expr = json_path_query_first_expr(col_expr, path).cast_as("text");
    let starts_with_expr = Expr::cust_with_exprs(
        "starts_with($1, $2)",
        [get_value_expr, Expr::val(value).into()],
    );

    type_assert_expr.and(starts_with_expr)
}

fn escape_like_expr(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn json_path_ends_with_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: String,
) -> SimpleExpr {
    json_path_like_expr(col_expr, path, format!("%{}", escape_like_expr(&value)))
}

fn json_path_str_between_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    lhs: String,
    rhs: String,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "string");
    let right = json_path_query_first_expr(col_expr, path)
        .cast_as("text")
        .between(lhs, rhs);

    left.and(right)
}

fn json_path_str_not_between_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    lhs: String,
    rhs: String,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "string");
    let right = json_path_query_first_expr(col_expr, path)
        .cast_as("text")
        .not_between(lhs, rhs);

    left.and(right)
}

fn json_path_num_between_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    lhs: Decimal,
    rhs: Decimal,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "number");
    let right = json_path_query_first_expr(col_expr, path)
        .cast_as("numeric")
        .between(lhs, rhs);

    left.and(right)
}

fn json_path_num_not_between_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    lhs: Decimal,
    rhs: Decimal,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "number");
    let right = json_path_query_first_expr(col_expr, path)
        .cast_as("numeric")
        .not_between(lhs, rhs);

    left.and(right)
}

fn json_path_bool_between_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    lhs: bool,
    rhs: bool,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "boolean");
    let right = json_path_query_first_expr(col_expr, path)
        .cast_as("boolean")
        .between(lhs, rhs);

    left.and(right)
}

fn json_path_bool_not_between_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    lhs: bool,
    rhs: bool,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let left = json_path_type_assert_expr(col_expr.clone(), path, "boolean");
    let right = json_path_query_first_expr(col_expr, path)
        .cast_as("boolean")
        .not_between(lhs, rhs);

    left.and(right)
}

fn json_path_contains_expr(
    col_expr: impl Into<SimpleExpr>,
    path: &JsonPath,
    value: JsonValue,
) -> SimpleExpr {
    let col_expr = col_expr.into();
    let json_path_array_contains = Expr::cust_with_exprs(
        "jsonb_path_query_first($1, $2) @> $3",
        [
            col_expr.clone(),
            json_path_expr(path),
            Expr::val(DbValue::Json(Some(Box::new(JsonValue::Array(vec![
                value.clone(),
            ])))))
            .into(),
        ],
    );
    let mut case = Expr::case(
        Condition::all()
            .add(json_path_type_assert_expr(col_expr.clone(), path, "array"))
            .add(json_path_array_contains),
        Expr::cust("true"),
    );

    if let JsonValue::String(s) = value {
        let json_path_str_contains = json_path_query_first_expr(col_expr.clone(), path)
            .cast_as("text")
            .like(format!("%{}%", escape_like_expr(&s)));

        case = case.case(
            Condition::all()
                .add(json_path_type_assert_expr(col_expr, path, "string"))
                .add(json_path_str_contains),
            Expr::cust("true"),
        )
    };

    case.finally(Expr::cust("false")).eq(Expr::cust("true"))
}

fn prepare_json_leaf_condition(
    col_expr: impl Into<SimpleExpr>,
    op: JsonFilterOperation,
    value: JsonValue,
    path: &JsonPath,
) -> RecorderResult<SimpleExpr> {
    Ok(match (op, value) {
        (op @ (JsonFilterOperation::Exists | JsonFilterOperation::NotExists), value) => match value
        {
            JsonValue::Bool(exists) => {
                let json_exists_expr = json_path_exists_expr(col_expr, path);
                if (op == JsonFilterOperation::Exists && exists)
                    || (op == JsonFilterOperation::NotExists && !exists)
                {
                    json_exists_expr
                } else {
                    json_exists_expr.not()
                }
            }
            _ => Err(SchemaError(
                "JsonFilterInput leaf can not be $exists or $not_exists with a non-boolean value"
                    .to_string(),
            ))?,
        },
        (
            JsonFilterOperation::And
            | JsonFilterOperation::Or
            | JsonFilterOperation::JsonbPathQuery
            | JsonFilterOperation::Any
            | JsonFilterOperation::Not,
            _,
        ) => {
            unreachable!("JsonFilterInput leaf can not be $and or $or with any value")
        }
        (JsonFilterOperation::Equals, value) => json_path_eq_expr(col_expr, path, value),
        (JsonFilterOperation::NotEquals, value) => json_path_ne_expr(col_expr, path, value),
        (op @ (JsonFilterOperation::IsIn | JsonFilterOperation::IsNotIn), value) => {
            if let JsonValue::Array(values) = value {
                let expr = json_path_is_in_values_expr(col_expr, path, values.clone());
                if op == JsonFilterOperation::IsIn {
                    expr
                } else {
                    expr.not()
                }
            } else {
                Err(SchemaError(
                    "JsonFilterInput leaf can not be $is_in or $is_not_in with a non-array value"
                        .to_string(),
                ))?
            }
        }
        (JsonFilterOperation::IsNull, value) => match value {
            JsonValue::Bool(is) => {
                let expr = json_path_is_null_expr(col_expr, path);
                if is { expr } else { expr.not() }
            }
            _ => Err(SchemaError(
                "JsonFilterInput leaf can not be $is_null with a non-boolean value".to_string(),
            ))?,
        },
        (JsonFilterOperation::IsNotNull, value) => match value {
            JsonValue::Bool(is) => {
                let expr = json_path_is_not_null_expr(col_expr, path);
                if is { expr } else { expr.not() }
            }
            _ => Err(SchemaError(
                "JsonFilterInput leaf can not be $is_not_null with a non-boolean value".to_string(),
            ))?,
        },
        (JsonFilterOperation::Contains, value) => json_path_contains_expr(col_expr, path, value),
        (
            op @ (JsonFilterOperation::GreaterThan
            | JsonFilterOperation::LessThan
            | JsonFilterOperation::GreaterThanEquals
            | JsonFilterOperation::LessThanEquals),
            value,
        ) => {
            let lexpr = json_path_query_first_auto_cast_expr(col_expr, path, &value)?;
            let rexpr: SimpleExpr = match value {
                JsonValue::Number(n) => Expr::val(DbValue::Decimal(Some(Box::new(
                    convert_json_number_to_db_decimal(n)?,
                ))))
                .into(),
                JsonValue::Bool(b) => Expr::val(b).into(),
                JsonValue::String(s) => Expr::val(s).into(),
                _ => Err(SchemaError(format!(
                    "JsonFilterInput leaf can not be {} with an array, object or null",
                    op.as_ref()
                )))?,
            };
            match op {
                JsonFilterOperation::GreaterThan => lexpr.gt(rexpr),
                JsonFilterOperation::GreaterThanEquals => lexpr.gte(rexpr),
                JsonFilterOperation::LessThan => lexpr.lt(rexpr),
                JsonFilterOperation::LessThanEquals => lexpr.lte(rexpr),
                _ => unreachable!(),
            }
        }
        (JsonFilterOperation::StartsWith, value) => {
            if let JsonValue::String(s) = value {
                json_path_starts_with_expr(col_expr, path, s)
            } else {
                Err(SchemaError(
                    "JsonFilterInput leaf can not be $starts_with with a non-string value"
                        .to_string(),
                ))?
            }
        }
        (JsonFilterOperation::EndsWith, value) => {
            if let JsonValue::String(s) = value {
                json_path_ends_with_expr(col_expr, path, s)
            } else {
                Err(SchemaError(
                    "JsonFilterInput leaf can not be $ends_with with a non-string value"
                        .to_string(),
                ))?
            }
        }
        (JsonFilterOperation::Like, value) => {
            if let JsonValue::String(s) = value {
                json_path_like_expr(col_expr, path, s)
            } else {
                Err(SchemaError(
                    "JsonFilterInput leaf can not be $like with a non-string value".to_string(),
                ))?
            }
        }
        (JsonFilterOperation::NotLike, value) => {
            if let JsonValue::String(s) = value {
                json_path_not_like_expr(col_expr, path, s)
            } else {
                Err(SchemaError(
                    "JsonFilterInput leaf can not be $not_like with a non-string value".to_string(),
                ))?
            }
        }
        (op @ (JsonFilterOperation::Between | JsonFilterOperation::NotBetween), value) => {
            if let JsonValue::Array(mut values) = value {
                if values.len() != 2 {
                    return Err(SchemaError(
                        "JsonFilterInput leaf can not be $between or $not_between with a \
                         non-array value"
                            .to_string(),
                    )
                    .into());
                } else {
                    let (rhs, lhs) = (values.pop().unwrap(), values.pop().unwrap());
                    match (lhs, rhs) {
                        (JsonValue::Number(lhs), JsonValue::Number(rhs)) => {
                            let (lhs, rhs) = (
                                convert_json_number_to_db_decimal(lhs)?,
                                convert_json_number_to_db_decimal(rhs)?,
                            );
                            if op == JsonFilterOperation::Between {
                                json_path_num_between_expr(col_expr, path, lhs, rhs)
                            } else {
                                json_path_num_not_between_expr(col_expr, path, lhs, rhs)
                            }
                        }
                        (JsonValue::String(lhs), JsonValue::String(rhs)) => {
                            if op == JsonFilterOperation::Between {
                                json_path_str_between_expr(col_expr, path, lhs, rhs)
                            } else {
                                json_path_str_not_between_expr(col_expr, path, lhs, rhs)
                            }
                        }
                        (JsonValue::Bool(lhs), JsonValue::Bool(rhs)) => {
                            if op == JsonFilterOperation::Between {
                                json_path_bool_between_expr(col_expr, path, lhs, rhs)
                            } else {
                                json_path_bool_not_between_expr(col_expr, path, lhs, rhs)
                            }
                        }
                        _ => Err(SchemaError(
                            "JsonFilterInput leaf can not be $between without two same type \
                             number, string or boolean value"
                                .to_string(),
                        ))?,
                    }
                }
            } else {
                Err(SchemaError(
                    "JsonFilterInput leaf can not be $between with a non-array value".to_string(),
                ))?
            }
        }
    })
}

fn recursive_prepare_json_node_condition<E>(
    expr: &E,
    node: JsonValue,
    mut path: JsonPath,
) -> RecorderResult<(Condition, JsonPath)>
where
    E: Into<SimpleExpr> + Clone,
{
    enum JsonIndex {
        Str(String),
        Num(u64),
    }

    impl TryFrom<JsonIndex> for JsonPathSegment {
        type Error = SchemaError;

        fn try_from(index: JsonIndex) -> Result<Self, Self::Error> {
            match index {
                JsonIndex::Str(s) => s.try_into(),
                JsonIndex::Num(n) => n.try_into(),
            }
        }
    }

    let map: Vec<(JsonIndex, JsonValue)> = match node {
        JsonValue::Object(object) => object
            .into_iter()
            .map(|(k, v)| (JsonIndex::Str(k), v))
            .collect(),
        JsonValue::Array(array) => array
            .into_iter()
            .enumerate()
            .map(|(i, v)| (JsonIndex::Num(i as u64), v))
            .collect(),
        _ => Err(SchemaError(format!(
            "Json filter input node must be an object or array, but got {node}"
        )))?,
    };
    let mut conditions = Condition::all();

    for (key, mut value) in map {
        if let JsonIndex::Str(str_key) = &key
            && let Some(operation) = JsonFilterOperation::parse_str(str_key)?
        {
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
                    let values = {
                        let a = value
                            .as_array_mut()
                            .and_then(|arr| if arr.len() >= 2 { Some(arr) } else { None })
                            .ok_or(SchemaError(
                                "$or operation must be an array of at least two sub filters"
                                    .to_string(),
                            ))?;
                        let mut b = vec![];
                        std::mem::swap(a, &mut b);
                        b
                    };

                    for value in values {
                        let (c, rpath) = recursive_prepare_json_node_condition(expr, value, path)?;
                        condition = condition.add(c);
                        path = rpath;
                    }

                    conditions = conditions.add(condition);
                }
                JsonFilterOperation::JsonbPathQuery => {
                    path.push(JsonPathSegment::JsonbPathQuery(
                        str_key.split_at(7).1.to_string(),
                    ))?;
                    let (condition, rpath) =
                        recursive_prepare_json_node_condition(expr, value, path)?;
                    conditions = conditions.add(condition);
                    path = rpath;
                    path.pop();
                }
                JsonFilterOperation::Any => {
                    continue;
                }
                JsonFilterOperation::Not => {
                    let (condition, rpath) =
                        recursive_prepare_json_node_condition(expr, value, path)?;
                    conditions = conditions.add(condition.not());
                    path = rpath;
                }
                op => {
                    let condition = prepare_json_leaf_condition(expr.clone(), op, value, &path)?;
                    conditions = conditions.add(condition);
                }
            }
        } else {
            let segment: JsonPathSegment = key.try_into()?;
            path.push(segment)?;
            let result = recursive_prepare_json_node_condition(expr, value, path)?;
            conditions = conditions.add(result.0);
            path = result.1;
            path.pop();
        }
    }

    Ok((conditions, path))
}

pub fn prepare_json_filter_input<E>(expr: &E, value: JsonValue) -> RecorderResult<Condition>
where
    E: Into<SimpleExpr> + Clone,
{
    let (condition, _) = recursive_prepare_json_node_condition(expr, value, JsonPath::new())?;

    Ok(condition)
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

pub static JSONB_FILTER_INFO: OnceCell<FilterInfo> = OnceCell::new();

pub fn jsonb_filter_condition_function<T>(
    _context: &BuilderContext,
    column: &T::Column,
) -> FnFilterCondition
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let column = *column;
    Box::new(move |mut condition, filter| {
        let filter_value = to_value(filter.as_index_map())
            .map_err(|e| SeaographyError::AsyncGraphQLError(GraphqlError::new_with_source(e)))?;

        let filter = JsonFilterInput::parse(filter_value)
            .map_err(|e| SeaographyError::AsyncGraphQLError(GraphqlError::new(format!("{e:?}"))))?;

        let cond_where = prepare_json_filter_input(&Expr::col(column), filter.0)
            .map_err(|e| SeaographyError::AsyncGraphQLError(GraphqlError::new_with_source(e)))?;

        condition = condition.add(cond_where);
        Ok(condition)
    })
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use sea_orm::{
        DeriveIden,
        sea_query::{IntoCondition, PostgresQueryBuilder, Query, Value, Values},
    };
    use serde_json::json;

    use super::*;
    use crate::errors::{RecorderError, RecorderResult};

    #[derive(DeriveIden)]
    enum TestTable {
        Table,
        Job,
    }

    fn build_test_query_sql(condition: impl IntoCondition) -> (String, Vec<Value>) {
        let (sql, Values(values)) = Query::select()
            .column(TestTable::Job)
            .cond_where(condition)
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
    fn test_json_path_exists_expr() {
        let (sql, params) = build_test_query_sql(json_path_exists_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE jsonb_path_exists(\"test_table\".\"job\", \
             $1)"
        );
        assert_eq!(params[0], "$.a.b.c".into());
    }

    #[test]
    fn test_json_path_is_in_expr() -> RecorderResult<()> {
        let (sql, params) = build_test_query_sql(json_path_is_in_values_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            vec![json!(1), json!("str"), json!(true)],
        ));

        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE \
             jsonb_path_query_first(\"test_table\".\"job\", $1) = ANY($2)"
        );
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], DbValue::String(Some(Box::new("$.a.b.c".into()))));
        assert_matches!(params[1], DbValue::Array(..));

        Ok(())
    }

    #[test]
    fn test_json_path_eq_expr() -> RecorderResult<()> {
        let (sql, params) = build_test_query_sql(json_path_eq_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            json!("str"),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE \
             (jsonb_path_query_first(\"test_table\".\"job\", $1)) = $2"
        );
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], DbValue::String(Some(Box::new("$.a.b.c".into()))));
        assert_eq!(params[1], DbValue::Json(Some(Box::new(json!("str")))));

        Ok(())
    }

    #[test]
    fn test_json_path_type_assert_expr() {
        let (sql, _) = build_test_query_sql(json_path_type_assert_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            "string",
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE jsonb_path_exists(\"test_table\".\"job\", \
             $1 || ' ? (@.type() = \"string\")')"
        );
    }

    #[test]
    fn test_json_path_contains_expr() {
        {
            let (sql, params) = build_test_query_sql(json_path_contains_expr(
                Expr::col((TestTable::Table, TestTable::Job)),
                &build_test_json_path(&["a", "b", "c"]),
                json!(1),
            ));

            assert_eq!(
                sql,
                "SELECT \"job\" FROM \"test_table\" WHERE (CASE WHEN \
                 ((jsonb_path_exists(\"test_table\".\"job\", $1 || ' ? (@.type() = \"array\")')) \
                 AND (jsonb_path_query_first(\"test_table\".\"job\", $2) @> $3)) THEN true ELSE \
                 false END) = (true)"
            );
            assert_eq!(params.len(), 3);
            assert_eq!(params[0], "$.a.b.c".into());
            assert_eq!(params[1], "$.a.b.c".into());
            assert_eq!(params[2], json!([1]).into());
        }
        {
            let (sql, params) = build_test_query_sql(json_path_contains_expr(
                Expr::col((TestTable::Table, TestTable::Job)),
                &build_test_json_path(&["a", "b", "c"]),
                json!("str"),
            ));

            assert_eq!(
                sql,
                "SELECT \"job\" FROM \"test_table\" WHERE (CASE WHEN \
                 ((jsonb_path_exists(\"test_table\".\"job\", $1 || ' ? (@.type() = \"array\")')) \
                 AND (jsonb_path_query_first(\"test_table\".\"job\", $2) @> $3)) THEN true WHEN \
                 ((jsonb_path_exists(\"test_table\".\"job\", $4 || ' ? (@.type() = \"string\")')) \
                 AND CAST((jsonb_path_query_first(\"test_table\".\"job\", $5)) AS text) LIKE $6) \
                 THEN true ELSE false END) = (true)"
            );
            assert_eq!(params.len(), 6);
            assert_eq!(params[0], "$.a.b.c".into());
            assert_eq!(params[1], "$.a.b.c".into());
            assert_eq!(params[2], json!(["str"]).into());
            assert_eq!(params[3], "$.a.b.c".into());
            assert_eq!(params[4], "$.a.b.c".into());
            assert_eq!(params[5], "%str%".into());
        }
    }

    #[test]
    fn test_json_path_between_expr() {
        {
            let (sql, params) = build_test_query_sql(json_path_num_between_expr(
                Expr::col((TestTable::Table, TestTable::Job)),
                &build_test_json_path(&["a", "b", "c"]),
                Decimal::from(1),
                Decimal::from(2),
            ));
            assert_eq!(
                sql,
                "SELECT \"job\" FROM \"test_table\" WHERE \
                 (jsonb_path_exists(\"test_table\".\"job\", $1 || ' ? (@.type() = \"number\")')) \
                 AND (CAST((jsonb_path_query_first(\"test_table\".\"job\", $2)) AS numeric) \
                 BETWEEN $3 AND $4)"
            );
            assert_eq!(params.len(), 4);
            assert_eq!(params[0], "$.a.b.c".into());
            assert_eq!(params[1], "$.a.b.c".into());
            assert_eq!(params[2], Decimal::from(1).into());
            assert_eq!(params[3], Decimal::from(2).into());
        }
        {
            let (sql, params) = build_test_query_sql(json_path_str_between_expr(
                Expr::col((TestTable::Table, TestTable::Job)),
                &build_test_json_path(&["a", "b", "c"]),
                "1".into(),
                "2".into(),
            ));
            assert_eq!(
                sql,
                "SELECT \"job\" FROM \"test_table\" WHERE \
                 (jsonb_path_exists(\"test_table\".\"job\", $1 || ' ? (@.type() = \"string\")')) \
                 AND (CAST((jsonb_path_query_first(\"test_table\".\"job\", $2)) AS text) BETWEEN \
                 $3 AND $4)"
            );
            assert_eq!(params.len(), 4);
            assert_eq!(params[0], "$.a.b.c".into());
            assert_eq!(params[1], "$.a.b.c".into());
            assert_eq!(params[2], "1".into());
            assert_eq!(params[3], "2".into());
        }
        {
            let (sql, params) = build_test_query_sql(json_path_bool_between_expr(
                Expr::col((TestTable::Table, TestTable::Job)),
                &build_test_json_path(&["a", "b", "c"]),
                true,
                false,
            ));
            assert_eq!(
                sql,
                "SELECT \"job\" FROM \"test_table\" WHERE \
                 (jsonb_path_exists(\"test_table\".\"job\", $1 || ' ? (@.type() = \"boolean\")')) \
                 AND (CAST((jsonb_path_query_first(\"test_table\".\"job\", $2)) AS boolean) \
                 BETWEEN $3 AND $4)"
            );
            assert_eq!(params.len(), 4);
            assert_eq!(params[0], "$.a.b.c".into());
            assert_eq!(params[1], "$.a.b.c".into());
            assert_eq!(params[2], true.into());
            assert_eq!(params[3], false.into());
        }
    }

    #[test]
    fn test_json_path_ends_with_expr() {
        let (sql, params) = build_test_query_sql(json_path_ends_with_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            "str%".into(),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE (jsonb_path_exists(\"test_table\".\"job\", \
             $1 || ' ? (@.type() = \"string\")')) AND \
             CAST((jsonb_path_query_first(\"test_table\".\"job\", $2)) AS text) LIKE $3"
        );
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], "$.a.b.c".into());
        assert_eq!(params[1], "$.a.b.c".into());
        assert_eq!(params[2], "%str\\%".into());
    }

    #[test]
    fn test_json_path_starts_with_expr() {
        let (sql, params) = build_test_query_sql(json_path_starts_with_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            "%str%".into(),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE (jsonb_path_exists(\"test_table\".\"job\", \
             $1 || ' ? (@.type() = \"string\")')) AND \
             (starts_with(CAST((jsonb_path_query_first(\"test_table\".\"job\", $2)) AS text), $3))"
        );
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], "$.a.b.c".into());
        assert_eq!(params[1], "$.a.b.c".into());
        assert_eq!(params[2], "%str%".into());
    }

    #[test]
    fn test_json_path_like_expr() {
        let (sql, params) = build_test_query_sql(json_path_like_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
            "%str%",
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE (jsonb_path_exists(\"test_table\".\"job\", \
             $1 || ' ? (@.type() = \"string\")')) AND \
             CAST((jsonb_path_query_first(\"test_table\".\"job\", $2)) AS text) LIKE $3"
        );
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], "$.a.b.c".into());
        assert_eq!(params[1], "$.a.b.c".into());
        assert_eq!(params[2], "%str%".into());
    }

    #[test]
    fn test_json_path_is_not_null_expr() {
        let (sql, params) = build_test_query_sql(json_path_is_not_null_expr(
            Expr::col((TestTable::Table, TestTable::Job)),
            &build_test_json_path(&["a", "b", "c"]),
        ));
        assert_eq!(
            sql,
            "SELECT \"job\" FROM \"test_table\" WHERE jsonb_path_exists(\"test_table\".\"job\", \
             $1 || ' ? (@ != null)')"
        );
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "$.a.b.c".into());
    }

    #[test]
    fn test_convert_json_number_to_db_decimal() {
        assert_eq!(
            convert_json_number_to_db_decimal(
                serde_json::Number::from_f64(1.234_567_890_123_456_7).unwrap()
            )
            .unwrap(),
            Decimal::from_f64(1.234_567_890_123_456_7).unwrap()
        );
        assert_eq!(
            convert_json_number_to_db_decimal(serde_json::Number::from(9007199254740991i64))
                .unwrap(),
            Decimal::from(9007199254740991i64)
        );
    }

    #[test]
    fn test_prepare_json_filter_input() -> RecorderResult<()> {
        {
            let condition = prepare_json_filter_input(
                &Expr::col((TestTable::Table, TestTable::Job)),
                json!({ "a": { "b": { "c": 1 } } }),
            );

            assert_matches!(condition, Err(RecorderError::GraphQLSchemaError { .. }));
        }
        {
            let condition = prepare_json_filter_input(
                &Expr::col((TestTable::Table, TestTable::Job)),
                json!({ "$and": [
                    {
                        "$or": [
                            {
                                "a": {
                                  "b": {
                                    "$eq": 1
                                  }
                                }
                            },
                            {
                                "$not": {
                                    "$query:.c.d.e": {
                                        "$is_in": [1, "haha", true]
                                    }
                                }
                            }
                        ]
                    },
                    {
                        "d": [
                            {
                                "$any": true
                            },
                            {
                                "$eq": [1, 2, 3]
                            }
                        ]
                    }
                ] }),
            )?;

            let (sql, params) = build_test_query_sql(condition);
            assert_eq!(
                sql,
                "SELECT \"job\" FROM \"test_table\" WHERE \
                 ((jsonb_path_query_first(\"test_table\".\"job\", $1)) = $2 OR (NOT \
                 (jsonb_path_query_first(\"test_table\".\"job\", $3) = ANY($4)))) AND (TRUE AND \
                 (jsonb_path_query_first(\"test_table\".\"job\", $5)) = $6)"
            );
            assert_eq!(params.len(), 6);
            assert_eq!(params[0], "$.a.b".into());
            assert_eq!(params[1], json!(1).into());
            assert_eq!(params[2], "$.c.d.e".into());
            assert_matches!(params[3], DbValue::Array(ArrayType::Json, ..));
            assert_eq!(params[4], "$.d[1]".into());
            assert_eq!(params[5], json!([1, 2, 3]).into());
        }

        Ok(())
    }
}
