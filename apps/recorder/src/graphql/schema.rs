use std::sync::Arc;

use async_graphql::dynamic::*;
use once_cell::sync::OnceCell;
use seaography::{Builder, BuilderContext};

use crate::{
    app::AppContextTrait,
    graphql::{
        domains::{
            credential_3rd::register_credential3rd_to_schema_builder,
            crypto::register_crypto_to_schema_context,
            subscriber_tasks::{
                register_subscriber_tasks_to_schema_builder,
                register_subscriber_tasks_to_schema_context,
            },
            subscribers::{
                register_subscribers_to_schema_builder, register_subscribers_to_schema_context,
                restrict_subscriber_for_entity,
            },
            subscriptions::register_subscriptions_to_schema_builder,
        },
        infra::json::register_jsonb_input_filter_to_schema_builder,
    },
};

pub static CONTEXT: OnceCell<BuilderContext> = OnceCell::new();

pub fn build_schema(
    app_ctx: Arc<dyn AppContextTrait>,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    use crate::models::*;
    let database = app_ctx.db().as_ref().clone();

    let context = CONTEXT.get_or_init(|| {
        let mut context = BuilderContext::default();

        {
            // domains
            register_subscribers_to_schema_context(&mut context);

            {
                restrict_subscriber_for_entity::<downloaders::Entity>(
                    &mut context,
                    &downloaders::Column::SubscriberId,
                );
                restrict_subscriber_for_entity::<downloads::Entity>(
                    &mut context,
                    &downloads::Column::SubscriberId,
                );
                restrict_subscriber_for_entity::<episodes::Entity>(
                    &mut context,
                    &episodes::Column::SubscriberId,
                );
                restrict_subscriber_for_entity::<subscriptions::Entity>(
                    &mut context,
                    &subscriptions::Column::SubscriberId,
                );
                restrict_subscriber_for_entity::<subscribers::Entity>(
                    &mut context,
                    &subscribers::Column::Id,
                );
                restrict_subscriber_for_entity::<subscription_bangumi::Entity>(
                    &mut context,
                    &subscription_bangumi::Column::SubscriberId,
                );
                restrict_subscriber_for_entity::<subscription_episode::Entity>(
                    &mut context,
                    &subscription_episode::Column::SubscriberId,
                );
                restrict_subscriber_for_entity::<credential_3rd::Entity>(
                    &mut context,
                    &credential_3rd::Column::SubscriberId,
                );
            }

            register_crypto_to_schema_context(&mut context, app_ctx.clone());
            register_subscriber_tasks_to_schema_context(&mut context);
        }
        context
    });

    let mut builder = Builder::new(context, database.clone());

    {
        // infra
        builder = register_jsonb_input_filter_to_schema_builder(builder);
    }
    {
        // domains
        builder = register_subscribers_to_schema_builder(builder);

        seaography::register_entities!(
            builder,
            [
                bangumi,
                downloaders,
                downloads,
                episodes,
                subscription_bangumi,
                subscription_episode,
                subscriptions,
                credential_3rd
            ]
        );

        {
            builder.register_enumeration::<downloads::DownloadStatus>();
            builder.register_enumeration::<subscriptions::SubscriptionCategory>();
            builder.register_enumeration::<downloaders::DownloaderCategory>();
            builder.register_enumeration::<downloads::DownloadMime>();
            builder.register_enumeration::<credential_3rd::Credential3rdType>();
        }

        builder = register_subscriptions_to_schema_builder(builder);
        builder = register_credential3rd_to_schema_builder(builder);
        builder = register_subscriber_tasks_to_schema_builder(builder);
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
        .data(app_ctx)
        .finish()
        .inspect_err(|e| tracing::error!(e = ?e))
}
