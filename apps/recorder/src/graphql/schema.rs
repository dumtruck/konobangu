use std::sync::Arc;

use async_graphql::dynamic::*;
use once_cell::sync::OnceCell;
use seaography::{Builder, BuilderContext};

use crate::{
    app::AppContextTrait,
    graphql::{
        domains::{
            bangumi::{register_bangumi_to_schema_builder, register_bangumi_to_schema_context},
            credential_3rd::{
                register_credential3rd_to_schema_builder, register_credential3rd_to_schema_context,
            },
            cron::{register_cron_to_schema_builder, register_cron_to_schema_context},
            downloaders::{
                register_downloaders_to_schema_builder, register_downloaders_to_schema_context,
            },
            downloads::{
                register_downloads_to_schema_builder, register_downloads_to_schema_context,
            },
            episodes::{register_episodes_to_schema_builder, register_episodes_to_schema_context},
            feeds::{register_feeds_to_schema_builder, register_feeds_to_schema_context},
            subscriber_tasks::{
                register_subscriber_tasks_to_schema_builder,
                register_subscriber_tasks_to_schema_context,
            },
            subscribers::{
                register_subscribers_to_schema_builder, register_subscribers_to_schema_context,
            },
            subscription_bangumi::{
                register_subscription_bangumi_to_schema_builder,
                register_subscription_bangumi_to_schema_context,
            },
            subscription_episode::{
                register_subscription_episode_to_schema_builder,
                register_subscription_episode_to_schema_context,
            },
            subscriptions::{
                register_subscriptions_to_schema_builder, register_subscriptions_to_schema_context,
            },
        },
        infra::{
            json::register_jsonb_input_filter_to_schema_builder,
            name::{
                renormalize_data_field_names_to_schema_context,
                renormalize_filter_field_names_to_schema_context,
            },
        },
    },
};

pub static CONTEXT: OnceCell<BuilderContext> = OnceCell::new();

pub fn build_schema(
    app_ctx: Arc<dyn AppContextTrait>,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let database = app_ctx.db().as_ref().clone();

    let context = CONTEXT.get_or_init(|| {
        let mut context = BuilderContext::default();

        renormalize_filter_field_names_to_schema_context(&mut context);
        renormalize_data_field_names_to_schema_context(&mut context);

        {
            // domains
            register_feeds_to_schema_context(&mut context);
            register_subscribers_to_schema_context(&mut context);
            register_subscriptions_to_schema_context(&mut context);
            register_subscriber_tasks_to_schema_context(&mut context);
            register_credential3rd_to_schema_context(&mut context, app_ctx.clone());
            register_downloaders_to_schema_context(&mut context);
            register_downloads_to_schema_context(&mut context);
            register_episodes_to_schema_context(&mut context);
            register_subscription_bangumi_to_schema_context(&mut context);
            register_subscription_episode_to_schema_context(&mut context);
            register_bangumi_to_schema_context(&mut context);
            register_cron_to_schema_context(&mut context);
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
        builder = register_feeds_to_schema_builder(builder);
        builder = register_episodes_to_schema_builder(builder);
        builder = register_subscription_bangumi_to_schema_builder(builder);
        builder = register_subscription_episode_to_schema_builder(builder);
        builder = register_downloaders_to_schema_builder(builder);
        builder = register_downloads_to_schema_builder(builder);
        builder = register_subscriptions_to_schema_builder(builder);
        builder = register_credential3rd_to_schema_builder(builder);
        builder = register_subscriber_tasks_to_schema_builder(builder);
        builder = register_bangumi_to_schema_builder(builder);
        builder = register_cron_to_schema_builder(builder);
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
