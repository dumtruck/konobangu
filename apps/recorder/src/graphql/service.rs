use async_graphql::dynamic::Schema;
use sea_orm::DatabaseConnection;

use super::{config::GraphQLConfig, schema_root};
use crate::errors::app_error::RResult;

#[derive(Debug)]
pub struct GraphQLService {
    pub schema: Schema,
}

impl GraphQLService {
    pub async fn from_config_and_database(
        config: GraphQLConfig,
        db: DatabaseConnection,
    ) -> RResult<Self> {
        let schema = schema_root::schema(
            db,
            config.depth_limit.and_then(|l| l.into()),
            config.complexity_limit.and_then(|l| l.into()),
        )?;
        Ok(Self { schema })
    }
}
