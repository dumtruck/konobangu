use async_graphql::dynamic::{Schema, SchemaError};
use sea_orm::DatabaseConnection;

use super::{config::GraphQLConfig, schema_root};

#[derive(Debug)]
pub struct GraphQLService {
    pub schema: Schema,
}

impl GraphQLService {
    pub fn new(config: GraphQLConfig, db: DatabaseConnection) -> Result<Self, SchemaError> {
        let schema = schema_root::schema(db, config.depth_limit, config.complexity_limit)?;
        Ok(Self { schema })
    }
}
