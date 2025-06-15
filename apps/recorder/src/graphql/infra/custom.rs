use std::{pin::Pin, sync::Arc};

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputValue, ResolverContext, TypeRef, ValueAccessor,
};
use sea_orm::EntityTrait;
use seaography::{BuilderContext, EntityObjectBuilder, FilterInputBuilder, GuardAction};

use crate::{app::AppContextTrait, errors::RecorderResult};

pub type FilterMutationFn = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Option<ValueAccessor<'_>>,
        ) -> Pin<
            Box<dyn Future<Output = RecorderResult<Option<FieldValue<'a>>>> + Send + 'a>,
        > + Send
        + Sync,
>;

pub fn generate_entity_filter_mutation_field<T, N, R>(
    builder_context: &'static BuilderContext,
    field_name: N,
    type_ref: R,
    mutation_fn: FilterMutationFn,
) -> Field
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
    N: Into<String>,
    R: Into<TypeRef>,
{
    let entity_filter_input_builder = FilterInputBuilder {
        context: builder_context,
    };
    let entity_object_builder = EntityObjectBuilder {
        context: builder_context,
    };
    let object_name: String = entity_object_builder.type_name::<T>();

    let context = builder_context;

    let guard = builder_context.guards.entity_guards.get(&object_name);

    Field::new(field_name, type_ref, move |ctx| {
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

            let result = mutation_fn(&ctx, app_ctx.clone(), filters)
                .await
                .map_err(async_graphql::Error::new_with_source)?;

            Ok(result)
        })
    })
    .argument(InputValue::new(
        &context.entity_delete_mutation.filter_field,
        TypeRef::named(entity_filter_input_builder.type_name(&object_name)),
    ))
}
