use std::{iter::FusedIterator, pin::Pin, sync::Arc};

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputObject, InputValue, Object, ObjectAccessor,
    ResolverContext, TypeRef,
};
use sea_orm::{ActiveModelTrait, Condition, EntityTrait, IntoActiveModel};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, EntityCreateBatchMutationBuilder,
    EntityCreateOneMutationBuilder, EntityDeleteMutationBuilder, EntityInputBuilder,
    EntityObjectBuilder, EntityUpdateMutationBuilder, GuardAction, RelationBuilder,
    get_filter_conditions,
};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    graphql::infra::name::{
        get_entity_filter_input_type_name, get_entity_name,
        get_entity_renormalized_filter_field_name,
    },
};

pub type FilterMutationFn = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Condition,
        ) -> Pin<
            Box<dyn Future<Output = RecorderResult<Option<FieldValue<'a>>>> + Send + 'a>,
        > + Send
        + Sync,
>;

pub type CreateOneMutationFn<M> = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            ObjectAccessor<'_>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<M>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type CreateBatchMutationFn<M> = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Vec<ObjectAccessor<'_>>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<Vec<M>>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type UpdateMutationFn<M> = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Condition,
            ObjectAccessor<'_>,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<Vec<M>>> + Send + 'a>>
        + Send
        + Sync,
>;

pub type DeleteMutationFn = Arc<
    dyn for<'a> Fn(
            &ResolverContext<'a>,
            Arc<dyn AppContextTrait>,
            Condition,
        ) -> Pin<Box<dyn Future<Output = RecorderResult<u64>> + Send + 'a>>
        + Send
        + Sync,
>;

pub fn generate_entity_default_insert_input_object<T>(context: &BuilderContext) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    EntityInputBuilder::insert_input_object::<T>(context)
}

pub fn generate_entity_default_update_input_object<T>(context: &BuilderContext) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    EntityInputBuilder::update_input_object::<T>(context)
}

pub fn generate_entity_default_basic_entity_object<T>(context: Arc<BuilderContext>) -> Object
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    EntityObjectBuilder::basic_to_object::<T>(context)
}

pub fn generate_entity_input_object<T>(
    context: &'static BuilderContext,
    is_insert: bool,
) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    if is_insert {
        EntityInputBuilder::insert_input_object::<T>(context)
    } else {
        EntityInputBuilder::update_input_object::<T>(context)
    }
}

pub fn generate_entity_filtered_mutation_field<E, N, R>(
    builder_context: Arc<BuilderContext>,
    field_name: N,
    type_ref: R,
    mutation_fn: FilterMutationFn,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    N: Into<String>,
    R: Into<TypeRef>,
{
    let object_name: String = get_entity_name::<E>(&builder_context);

    let guard = builder_context
        .guards
        .entity_guards
        .get(&object_name)
        .cloned();

    let filter_input_value = InputValue::new(
        get_entity_renormalized_filter_field_name(),
        TypeRef::named(get_entity_filter_input_type_name::<E>(&builder_context)),
    );

    Field::new(field_name, type_ref, move |ctx| {
        let mutation_fn = mutation_fn.clone();
        let builder_context = builder_context.clone();
        let guard_flag = if let Some(guard) = guard.as_ref() {
            (*guard)(&ctx)
        } else {
            GuardAction::Allow
        };
        FieldFuture::new(async move {
            if let GuardAction::Block(reason) = guard_flag {
                return Err::<Option<_>, async_graphql::Error>(async_graphql::Error::new(
                    reason.unwrap_or("Entity guard triggered.".into()),
                ));
            }

            let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

            let filters = ctx.args.get(get_entity_renormalized_filter_field_name());

            let filters = get_filter_conditions::<E>(&ctx, &builder_context, filters);

            let result = mutation_fn(&ctx, app_ctx.clone(), filters)
                .await
                .map_err(async_graphql::Error::new_with_source)?;

            Ok(result)
        })
    })
    .argument(filter_input_value)
}

pub fn generate_entity_create_one_mutation_field<E>(
    builder_context: Arc<BuilderContext>,
    mutation_fn: CreateOneMutationFn<E::Model>,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    EntityCreateOneMutationBuilder::to_field_with_mutation_fn::<E>(
        builder_context.clone(),
        Arc::new(move |resolver_ctx, input_object| {
            let result = resolver_ctx
                .data::<Arc<dyn AppContextTrait>>()
                .map(|app_ctx| mutation_fn(&resolver_ctx, app_ctx.clone(), input_object));
            Box::pin(async move { result?.await.map_err(async_graphql::Error::new_with_source) })
        }),
    )
}

pub fn generate_entity_default_create_one_mutation_field<E, A>(
    builder_context: Arc<BuilderContext>,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    EntityCreateOneMutationBuilder::to_field::<E, A>(builder_context, active_model_hooks)
}

pub fn generate_entity_create_batch_mutation_field<E, ID>(
    builder_context: Arc<BuilderContext>,
    mutation_fn: CreateBatchMutationFn<E::Model>,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    EntityCreateBatchMutationBuilder::to_field_with_mutation_fn::<E>(
        builder_context,
        Arc::new(move |resolver_ctx, input_objects| {
            let result = resolver_ctx
                .data::<Arc<dyn AppContextTrait>>()
                .map(|app_ctx| mutation_fn(&resolver_ctx, app_ctx.clone(), input_objects));
            Box::pin(async move { result?.await.map_err(async_graphql::Error::new_with_source) })
        }),
    )
}

pub fn generate_entity_default_create_batch_mutation_field<E, A>(
    builder_context: Arc<BuilderContext>,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    EntityCreateBatchMutationBuilder::to_field::<E, A>(builder_context, active_model_hooks)
}

pub fn generate_entity_update_mutation_field<E>(
    builder_context: Arc<BuilderContext>,
    mutation_fn: UpdateMutationFn<E::Model>,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    EntityUpdateMutationBuilder::to_field_with_mutation_fn::<E>(
        builder_context.clone(),
        Arc::new(move |resolver_ctx, filters, input_object| {
            let result = resolver_ctx
                .data::<Arc<dyn AppContextTrait>>()
                .map(|app_ctx| {
                    mutation_fn(
                        &resolver_ctx,
                        app_ctx.clone(),
                        get_filter_conditions::<E>(&resolver_ctx, &builder_context, filters),
                        input_object,
                    )
                });
            Box::pin(async move { result?.await.map_err(async_graphql::Error::new_with_source) })
        }),
    )
}

pub fn generate_entity_default_update_mutation_field<E, A>(
    builder_context: Arc<BuilderContext>,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    EntityUpdateMutationBuilder::to_field::<E, A>(builder_context, active_model_hooks)
}

pub fn generate_entity_delete_mutation_field<E>(
    builder_context: Arc<BuilderContext>,
    mutation_fn: DeleteMutationFn,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    EntityDeleteMutationBuilder::to_field_with_mutation_fn::<E>(
        builder_context.clone(),
        Arc::new(move |resolver_ctx, filters| {
            let result = resolver_ctx
                .data::<Arc<dyn AppContextTrait>>()
                .map(|app_ctx| {
                    mutation_fn(
                        &resolver_ctx,
                        app_ctx.clone(),
                        get_filter_conditions::<E>(&resolver_ctx, &builder_context, filters),
                    )
                });
            Box::pin(async move { result?.await.map_err(async_graphql::Error::new_with_source) })
        }),
    )
}

pub fn generate_entity_default_delete_mutation_field<E, A>(
    builder_context: Arc<BuilderContext>,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    EntityDeleteMutationBuilder::to_field::<E, A>(builder_context, active_model_hooks)
}

pub fn register_entity_default_mutations<E, A>(
    mut builder: SeaographyBuilder,
    active_model_hooks: bool,
) -> SeaographyBuilder
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    let builder_context = &builder.context;
    builder
        .outputs
        .push(generate_entity_default_basic_entity_object::<E>(
            builder_context.clone(),
        ));

    builder.inputs.extend([
        generate_entity_default_insert_input_object::<E>(&builder.context),
        generate_entity_default_update_input_object::<E>(&builder.context),
    ]);

    builder.mutations.extend([
        generate_entity_default_create_one_mutation_field::<E, A>(
            builder_context.clone(),
            active_model_hooks,
        ),
        generate_entity_default_create_batch_mutation_field::<E, A>(
            builder_context.clone(),
            active_model_hooks,
        ),
        generate_entity_default_update_mutation_field::<E, A>(
            builder_context.clone(),
            active_model_hooks,
        ),
        generate_entity_default_delete_mutation_field::<E, A>(
            builder_context.clone(),
            active_model_hooks,
        ),
    ]);

    builder
}

pub(crate) fn register_entity_default_readonly_impl<T, RE, I>(
    mut builder: SeaographyBuilder,
    entity: T,
) -> SeaographyBuilder
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
    RE: sea_orm::Iterable<Iterator = I> + RelationBuilder,
    I: Iterator<Item = RE> + Clone + DoubleEndedIterator + ExactSizeIterator + FusedIterator,
{
    builder.register_entity::<T>(
        <RE as sea_orm::Iterable>::iter()
            .map(|rel| RelationBuilder::get_relation(&rel, builder.context.clone()))
            .collect(),
    );
    builder = builder.register_entity_dataloader_one_to_one(entity, tokio::spawn);
    builder = builder.register_entity_dataloader_one_to_many(entity, tokio::spawn);
    builder
}

pub(crate) fn register_entity_default_writable_impl<T, RE, A, I>(
    mut builder: SeaographyBuilder,
    entity: T,
    active_model_hooks: bool,
) -> SeaographyBuilder
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = T> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
    RE: sea_orm::Iterable<Iterator = I> + RelationBuilder,
    I: Iterator<Item = RE> + Clone + DoubleEndedIterator + ExactSizeIterator + FusedIterator,
{
    builder = register_entity_default_readonly_impl::<T, RE, I>(builder, entity);
    builder = register_entity_default_mutations::<T, A>(builder, active_model_hooks);
    builder
}

macro_rules! register_entity_default_readonly {
    ($builder:expr, $module_path:ident) => {
        $crate::graphql::infra::custom::register_entity_default_readonly_impl::<
            $module_path::Entity,
            $module_path::RelatedEntity,
            _,
        >($builder, $module_path::Entity)
    };
}

macro_rules! register_entity_default_writable {
    ($builder:expr, $module_path:ident, $active_model_hooks:expr) => {
        $crate::graphql::infra::custom::register_entity_default_writable_impl::<
            $module_path::Entity,
            $module_path::RelatedEntity,
            $module_path::ActiveModel,
            _,
        >($builder, $module_path::Entity, $active_model_hooks)
    };
}

pub(crate) use register_entity_default_readonly;
pub(crate) use register_entity_default_writable;
