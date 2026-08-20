use sea_orm::{EntityName, entity::prelude::*};
use sea_query::Iden;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "auth_provider", table_name = "protocol_state")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub hash: String,
  pub kind: String,
  pub data: Json,
  pub family: Option<String>,
  pub expires_at: i64,
  pub consumed: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
pub(crate) use Column as State;
pub fn schema_name() -> &'static str {
  Entity.schema_name().expect("provider state owns a schema")
}
pub(crate) fn table() -> String {
  format!("\"{}\".\"{}\"", schema_name(), Entity.table_name())
}
pub(crate) fn col(column: Column) -> String {
  format!("\"{}\"", column.to_string())
}
