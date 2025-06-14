use std::{pin::Pin, sync::Arc};

use async_graphql::dynamic::{
    Field, FieldFuture, InputValue, ResolverContext, TypeRef, ValueAccessor,
};
use sea_orm::EntityTrait;
use seaography::{
    BuilderContext, EntityDeleteMutationBuilder, EntityObjectBuilder, FilterInputBuilder,
    GuardAction,
};

use crate::{app::AppContextTrait, errors::RecorderResult};

pub type DeleteMutationFn = Arc<
    dyn Fn(
            &ResolverContext<'_>,
            Arc<dyn AppContextTrait>,
            Option<ValueAccessor<'_>>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<Option<i32>>> + Send>>
        + Send
        + Sync,
>;

pub fn generate_custom_entity_delete_mutation_field<T>(
    builder_context: &'static BuilderContext,
    mutation_fn: DeleteMutationFn,
) -> Field
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_filter_input_builder = FilterInputBuilder {
        context: builder_context,
    };
    let entity_object_builder = EntityObjectBuilder {
        context: builder_context,
    };
    let entity_delete_mutation_builder = EntityDeleteMutationBuilder {
        context: builder_context,
    };
    let object_name: String = entity_object_builder.type_name::<T>();

    let context = builder_context;

    let guard = builder_context.guards.entity_guards.get(&object_name);

    Field::new(
        entity_delete_mutation_builder.type_name::<T>(),
        TypeRef::named_nn(TypeRef::INT),
        move |ctx| {
            let mutation_fn = mutation_fn.clone();
            FieldFuture::new(async move {
                let guard_flag = if let Some(guard) = guard {
                    (*guard)(&ctx)
                } else {
                    GuardAction::Allow
                };

                if let GuardAction::Block(reason) = guard_flag {
                    return Err::<Option<_>, async_graphql::Error>(async_graphql::Error::new(
                        reason.unwrap_or("Entity guard triggered.".into()),
                    ));
                }

                let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

                let filters = ctx.args.get(&context.entity_delete_mutation.filter_field);

                let result = mutation_fn(&ctx, app_ctx.clone(), filters).await?;

                Ok(result.map(async_graphql::Value::from))
            })
        },
    )
    .argument(InputValue::new(
        &context.entity_delete_mutation.filter_field,
        TypeRef::named(entity_filter_input_builder.type_name(&object_name)),
    ))
}
