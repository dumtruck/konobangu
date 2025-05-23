use async_graphql::dynamic::Schema;
use sea_orm::DatabaseConnection;

use super::{build_schema, config::GraphQLConfig};
use crate::errors::RecorderResult;

#[derive(Debug)]
pub struct GraphQLService {
    pub schema: Schema,
}

impl GraphQLService {
    pub async fn from_config_and_database(
        config: GraphQLConfig,
        db: DatabaseConnection,
    ) -> RecorderResult<Self> {
        let schema = build_schema(
            db,
            config.depth_limit.and_then(|l| l.into()),
            config.complexity_limit.and_then(|l| l.into()),
        )?;
        Ok(Self { schema })
    }
}
