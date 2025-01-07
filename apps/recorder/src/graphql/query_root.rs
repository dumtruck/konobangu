use async_graphql::dynamic::*;
use sea_orm::DatabaseConnection;
use seaography::{Builder, BuilderContext};

lazy_static::lazy_static! { static ref CONTEXT : BuilderContext = BuilderContext :: default () ; }

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    use crate::models::*;
    let mut builder = Builder::new(&CONTEXT, database.clone());

    seaography::register_entities!(
        builder,
        [
            bangumi,
            downloaders,
            downloads,
            episodes,
            subscribers,
            subscription_bangumi,
            subscription_episode,
            subscriptions
        ]
    );

    {
        builder.register_enumeration::<downloads::DownloadStatus>();
        builder.register_enumeration::<subscriptions::SubscriptionCategory>();
        builder.register_enumeration::<downloaders::DownloaderCategory>();
        builder.register_enumeration::<downloads::DownloadMime>();
    }

    let schema = builder.schema_builder();
    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    schema
        .data(database)
        .finish()
        .inspect_err(|e| tracing::error!(e = ?e))
}
