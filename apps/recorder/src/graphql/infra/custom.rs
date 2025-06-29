use std::{iter::FusedIterator, pin::Pin, sync::Arc};

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputObject, InputValue, Object, ObjectAccessor,
    ResolverContext, TypeRef,
};
use sea_orm::{
    ActiveModelTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait,
};
use seaography::{
    Builder as SeaographyBuilder, BuilderContext, GuardAction, RelationBuilder,
    get_filter_conditions, prepare_active_model,
};

use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    graphql::infra::name::{
        get_entity_and_column_name_from_column_str, get_entity_basic_type_name,
        get_entity_create_batch_mutation_data_field_name,
        get_entity_create_batch_mutation_field_name,
        get_entity_create_one_mutation_data_field_name, get_entity_create_one_mutation_field_name,
        get_entity_delete_mutation_field_name, get_entity_delete_mutation_filter_field_name,
        get_entity_filter_input_type_name, get_entity_insert_data_input_type_name, get_entity_name,
        get_entity_renormalized_filter_field_name, get_entity_update_data_input_type_name,
        get_entity_update_mutation_data_field_name, get_entity_update_mutation_field_name,
        get_entity_update_mutation_filter_field_name,
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

pub fn generate_entity_default_insert_input_object<T>(
    builder_context: &'static BuilderContext,
) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_input_builder = seaography::EntityInputBuilder {
        context: builder_context,
    };

    entity_input_builder.insert_input_object::<T>()
}

pub fn generate_entity_default_update_input_object<T>(
    builder_context: &'static BuilderContext,
) -> InputObject
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_input_builder = seaography::EntityInputBuilder {
        context: builder_context,
    };

    entity_input_builder.update_input_object::<T>()
}

pub fn generate_entity_default_basic_entity_object<T>(
    builder_context: &'static BuilderContext,
) -> Object
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_object_builder = seaography::EntityObjectBuilder {
        context: builder_context,
    };

    entity_object_builder.basic_to_object::<T>()
}

pub fn generate_entity_filtered_mutation_field<E, N, R>(
    builder_context: &'static BuilderContext,
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
    let object_name: String = get_entity_name::<E>(builder_context);

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

            let filters = ctx.args.get(get_entity_renormalized_filter_field_name());

            let filters = get_filter_conditions::<E>(&ctx, builder_context, filters);

            let result = mutation_fn(&ctx, app_ctx.clone(), filters)
                .await
                .map_err(async_graphql::Error::new_with_source)?;

            Ok(result)
        })
    })
    .argument(InputValue::new(
        get_entity_renormalized_filter_field_name(),
        TypeRef::named(get_entity_filter_input_type_name::<E>(builder_context)),
    ))
}

pub fn generate_entity_create_one_mutation_field<E, ID>(
    builder_context: &'static BuilderContext,
    input_data_type_ref: Option<ID>,
    mutation_fn: CreateOneMutationFn<E::Model>,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    ID: Into<TypeRef>,
{
    let guard = builder_context
        .guards
        .entity_guards
        .get(&get_entity_name::<E>(builder_context));
    let field_guards = &builder_context.guards.field_guards;

    Field::new(
        get_entity_create_one_mutation_field_name::<E>(builder_context),
        TypeRef::named_nn(get_entity_basic_type_name::<E>(builder_context)),
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

                let value_accessor = ctx
                    .args
                    .get(get_entity_create_one_mutation_data_field_name(
                        builder_context,
                    ))
                    .unwrap();
                let input_object = value_accessor.object()?;

                for (column, _) in input_object.iter() {
                    let field_guard = field_guards.get(
                        &get_entity_and_column_name_from_column_str::<E>(builder_context, column),
                    );
                    let field_guard_flag = if let Some(field_guard) = field_guard {
                        (*field_guard)(&ctx)
                    } else {
                        GuardAction::Allow
                    };
                    if let GuardAction::Block(reason) = field_guard_flag {
                        return match reason {
                            Some(reason) => Err::<Option<_>, async_graphql::Error>(
                                async_graphql::Error::new(reason),
                            ),
                            None => Err::<Option<_>, async_graphql::Error>(
                                async_graphql::Error::new("Field guard triggered."),
                            ),
                        };
                    }
                }

                let result = mutation_fn(&ctx, app_ctx.clone(), input_object)
                    .await
                    .map_err(async_graphql::Error::new_with_source)?;

                Ok(Some(FieldValue::owned_any(result)))
            })
        },
    )
    .argument(InputValue::new(
        get_entity_create_one_mutation_data_field_name(builder_context),
        input_data_type_ref.map(|t| t.into()).unwrap_or_else(|| {
            TypeRef::named_nn(get_entity_insert_data_input_type_name::<E>(builder_context))
        }),
    ))
}

pub fn generate_entity_default_create_one_mutation_fn<T, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> CreateOneMutationFn<T::Model>
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = T> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    Arc::new(move |resolve_context, app_ctx, input_object| {
        let entity_input_builder = seaography::EntityInputBuilder {
            context: builder_context,
        };
        let entity_object_builder = seaography::EntityObjectBuilder {
            context: builder_context,
        };
        let active_model = prepare_active_model::<T, A>(
            &entity_input_builder,
            &entity_object_builder,
            &input_object,
            resolve_context,
        );

        Box::pin(async move {
            if active_model_hooks {
                let transaction = app_ctx.db().begin().await?;

                let active_model = active_model?;

                let active_model = active_model.before_save(&transaction, true).await?;

                let result: T::Model = active_model.insert(&transaction).await?;

                let result = A::after_save(result, &transaction, true).await?;

                transaction.commit().await?;

                Ok(result)
            } else {
                let db = app_ctx.db();

                let active_model = active_model?;

                let result: T::Model = active_model.insert(db).await?;

                Ok(result)
            }
        })
    })
}

pub fn generate_entity_default_create_one_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    generate_entity_create_one_mutation_field::<E, TypeRef>(
        builder_context,
        None,
        generate_entity_default_create_one_mutation_fn::<E, A>(builder_context, active_model_hooks),
    )
}

pub fn generate_entity_create_batch_mutation_field<E, ID>(
    builder_context: &'static BuilderContext,
    input_data_type_ref: Option<ID>,
    mutation_fn: CreateBatchMutationFn<E::Model>,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    ID: Into<TypeRef>,
{
    let object_name: String = get_entity_name::<E>(builder_context);
    let guard = builder_context.guards.entity_guards.get(&object_name);
    let field_guards = &builder_context.guards.field_guards;

    Field::new(
        get_entity_create_batch_mutation_field_name::<E>(builder_context),
        TypeRef::named_nn_list_nn(get_entity_basic_type_name::<E>(builder_context)),
        move |ctx| {
            let mutation_fn = mutation_fn.clone();
            FieldFuture::new(async move {
                let guard_flag = if let Some(guard) = guard {
                    (*guard)(&ctx)
                } else {
                    GuardAction::Allow
                };

                if let GuardAction::Block(reason) = guard_flag {
                    return match reason {
                        Some(reason) => Err::<Option<_>, async_graphql::Error>(
                            async_graphql::Error::new(reason),
                        ),
                        None => Err::<Option<_>, async_graphql::Error>(async_graphql::Error::new(
                            "Entity guard triggered.",
                        )),
                    };
                }

                let mut input_objects: Vec<ObjectAccessor<'_>> = vec![];
                let list = ctx
                    .args
                    .get(get_entity_create_batch_mutation_data_field_name(
                        builder_context,
                    ))
                    .unwrap()
                    .list()?;
                for input in list.iter() {
                    let input_object = input.object()?;
                    for (column, _) in input_object.iter() {
                        let field_guard =
                            field_guards.get(&get_entity_and_column_name_from_column_str::<E>(
                                builder_context,
                                column,
                            ));
                        let field_guard_flag = if let Some(field_guard) = field_guard {
                            (*field_guard)(&ctx)
                        } else {
                            GuardAction::Allow
                        };
                        if let GuardAction::Block(reason) = field_guard_flag {
                            return match reason {
                                Some(reason) => Err::<Option<_>, async_graphql::Error>(
                                    async_graphql::Error::new(reason),
                                ),
                                None => Err::<Option<_>, async_graphql::Error>(
                                    async_graphql::Error::new("Field guard triggered."),
                                ),
                            };
                        }
                    }

                    input_objects.push(input_object);
                }

                let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

                let results = mutation_fn(&ctx, app_ctx.clone(), input_objects)
                    .await
                    .map_err(async_graphql::Error::new_with_source)?;

                Ok(Some(FieldValue::list(
                    results.into_iter().map(FieldValue::owned_any),
                )))
            })
        },
    )
    .argument(InputValue::new(
        get_entity_create_batch_mutation_data_field_name(builder_context),
        input_data_type_ref.map(|t| t.into()).unwrap_or_else(|| {
            TypeRef::named_nn_list_nn(get_entity_insert_data_input_type_name::<E>(builder_context))
        }),
    ))
}

pub fn generate_entity_default_create_batch_mutation_fn<E, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> CreateBatchMutationFn<E::Model>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    Arc::new(move |resolve_context, app_ctx, input_objects| {
        let entity_input_builder = seaography::EntityInputBuilder {
            context: builder_context,
        };
        let entity_object_builder = seaography::EntityObjectBuilder {
            context: builder_context,
        };
        let active_models = input_objects
            .into_iter()
            .map(|input_object| {
                prepare_active_model::<E, A>(
                    &entity_input_builder,
                    &entity_object_builder,
                    &input_object,
                    resolve_context,
                )
            })
            .collect::<Result<Vec<_>, _>>();

        Box::pin(async move {
            if active_model_hooks {
                let transaction = app_ctx.db().begin().await?;

                let mut before_save_models = vec![];

                for active_model in active_models? {
                    let before_save_model = active_model.before_save(&transaction, false).await?;
                    before_save_models.push(before_save_model);
                }

                let models: Vec<E::Model> = E::insert_many(before_save_models)
                    .exec_with_returning_many(&transaction)
                    .await?;

                let mut result = vec![];
                for model in models {
                    let after_save_model = A::after_save(model, &transaction, false).await?;
                    result.push(after_save_model);
                }

                transaction.commit().await?;

                Ok(result)
            } else {
                let db = app_ctx.db();
                let active_models = active_models?;
                let results: Vec<E::Model> = E::insert_many(active_models)
                    .exec_with_returning_many(db)
                    .await?;

                Ok(results)
            }
        })
    })
}

pub fn generate_entity_default_create_batch_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    generate_entity_create_batch_mutation_field::<E, TypeRef>(
        builder_context,
        None,
        generate_entity_default_create_batch_mutation_fn::<E, A>(
            builder_context,
            active_model_hooks,
        ),
    )
}

pub fn generate_entity_update_mutation_field<E, I>(
    builder_context: &'static BuilderContext,
    input_data_type_ref: Option<I>,
    mutation_fn: UpdateMutationFn<E::Model>,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    I: Into<TypeRef>,
{
    let guard = builder_context
        .guards
        .entity_guards
        .get(&get_entity_name::<E>(builder_context));
    let field_guards = &builder_context.guards.field_guards;

    Field::new(
        get_entity_update_mutation_field_name::<E>(builder_context),
        TypeRef::named_nn_list_nn(get_entity_basic_type_name::<E>(builder_context)),
        move |ctx| {
            let mutation_fn = mutation_fn.clone();
            FieldFuture::new(async move {
                let guard_flag = if let Some(guard) = guard {
                    (*guard)(&ctx)
                } else {
                    GuardAction::Allow
                };

                if let GuardAction::Block(reason) = guard_flag {
                    return match reason {
                        Some(reason) => Err::<Option<_>, async_graphql::Error>(
                            async_graphql::Error::new(reason),
                        ),
                        None => Err::<Option<_>, async_graphql::Error>(async_graphql::Error::new(
                            "Entity guard triggered.",
                        )),
                    };
                }

                let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

                let filters = ctx.args.get(get_entity_update_mutation_filter_field_name(
                    builder_context,
                ));
                let filter_condition = get_filter_conditions::<E>(&ctx, builder_context, filters);

                let value_accessor = ctx
                    .args
                    .get(get_entity_update_mutation_data_field_name(builder_context))
                    .unwrap();
                let input_object = value_accessor.object()?;

                for (column, _) in input_object.iter() {
                    let field_guard = field_guards.get(
                        &get_entity_and_column_name_from_column_str::<E>(builder_context, column),
                    );
                    let field_guard_flag = if let Some(field_guard) = field_guard {
                        (*field_guard)(&ctx)
                    } else {
                        GuardAction::Allow
                    };
                    if let GuardAction::Block(reason) = field_guard_flag {
                        return match reason {
                            Some(reason) => Err::<Option<_>, async_graphql::Error>(
                                async_graphql::Error::new(reason),
                            ),
                            None => Err::<Option<_>, async_graphql::Error>(
                                async_graphql::Error::new("Field guard triggered."),
                            ),
                        };
                    }
                }

                let result = mutation_fn(&ctx, app_ctx.clone(), filter_condition, input_object)
                    .await
                    .map_err(async_graphql::Error::new_with_source)?;

                Ok(Some(FieldValue::list(
                    result.into_iter().map(FieldValue::owned_any),
                )))
            })
        },
    )
    .argument(InputValue::new(
        get_entity_update_mutation_data_field_name(builder_context),
        input_data_type_ref.map(|t| t.into()).unwrap_or_else(|| {
            TypeRef::named_nn(get_entity_update_data_input_type_name::<E>(builder_context))
        }),
    ))
    .argument(InputValue::new(
        get_entity_update_mutation_filter_field_name(builder_context),
        TypeRef::named(get_entity_filter_input_type_name::<E>(builder_context)),
    ))
}

pub fn generate_entity_default_update_mutation_fn<T, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> UpdateMutationFn<T::Model>
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = T> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    Arc::new(
        move |resolve_context, app_ctx, filter_condition, input_object| {
            let entity_input_builder = seaography::EntityInputBuilder {
                context: builder_context,
            };
            let entity_object_builder = seaography::EntityObjectBuilder {
                context: builder_context,
            };

            let active_model = prepare_active_model::<T, A>(
                &entity_input_builder,
                &entity_object_builder,
                &input_object,
                resolve_context,
            );

            Box::pin(async move {
                if active_model_hooks {
                    let transaction = app_ctx.db().begin().await?;

                    let active_model = active_model?;

                    let active_model = active_model.before_save(&transaction, false).await?;

                    let models = T::update_many()
                        .set(active_model)
                        .filter(filter_condition.clone())
                        .exec_with_returning(&transaction)
                        .await?;
                    let mut result = vec![];

                    for model in models {
                        result.push(A::after_save(model, &transaction, false).await?);
                    }

                    transaction.commit().await?;

                    Ok(result)
                } else {
                    let db = app_ctx.db();

                    let active_model = active_model?;

                    let result = T::update_many()
                        .set(active_model)
                        .filter(filter_condition.clone())
                        .exec_with_returning(db)
                        .await?;

                    Ok(result)
                }
            })
        },
    )
}

pub fn generate_entity_default_update_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    generate_entity_update_mutation_field::<E, TypeRef>(
        builder_context,
        None,
        generate_entity_default_update_mutation_fn::<E, A>(builder_context, active_model_hooks),
    )
}

pub fn generate_entity_delete_mutation_field<E>(
    builder_context: &'static BuilderContext,
    mutation_fn: DeleteMutationFn,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    let object_name: String = get_entity_name::<E>(builder_context);
    let guard = builder_context.guards.entity_guards.get(&object_name);

    Field::new(
        get_entity_delete_mutation_field_name::<E>(builder_context),
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

                let filters = ctx.args.get(get_entity_delete_mutation_filter_field_name(
                    builder_context,
                ));
                let filter_condition = get_filter_conditions::<E>(&ctx, builder_context, filters);

                let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

                let res = mutation_fn(&ctx, app_ctx.clone(), filter_condition)
                    .await
                    .map_err(async_graphql::Error::new_with_source)?;

                Ok(Some(async_graphql::Value::from(res)))
            })
        },
    )
    .argument(InputValue::new(
        get_entity_delete_mutation_filter_field_name(builder_context),
        TypeRef::named(get_entity_filter_input_type_name::<E>(builder_context)),
    ))
}

pub fn generate_entity_default_delete_mutation_fn<E, A>(
    _builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> DeleteMutationFn
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    Arc::new(move |_resolve_context, app_ctx, filter_condition| {
        Box::pin(async move {
            if active_model_hooks {
                let transaction = app_ctx.db().begin().await?;

                let models: Vec<E::Model> = E::find()
                    .filter(filter_condition.clone())
                    .all(&transaction)
                    .await?;

                let mut active_models: Vec<A> = vec![];
                for model in models {
                    let active_model = model.into_active_model();
                    active_models.push(active_model.before_delete(&transaction).await?);
                }

                let result = E::delete_many()
                    .filter(filter_condition)
                    .exec(&transaction)
                    .await?;

                for active_model in active_models {
                    active_model.after_delete(&transaction).await?;
                }

                transaction.commit().await?;

                Ok(result.rows_affected)
            } else {
                let db = app_ctx.db();

                let result = E::delete_many().filter(filter_condition).exec(db).await?;

                Ok(result.rows_affected)
            }
        })
    })
}

pub fn generate_entity_default_delete_mutation_field<E, A>(
    builder_context: &'static BuilderContext,
    active_model_hooks: bool,
) -> Field
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <E as EntityTrait>::Model: IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + sea_orm::ActiveModelBehavior + std::marker::Send + 'static,
{
    generate_entity_delete_mutation_field::<E>(
        builder_context,
        generate_entity_default_delete_mutation_fn::<E, A>(builder_context, active_model_hooks),
    )
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
    builder
        .outputs
        .push(generate_entity_default_basic_entity_object::<E>(
            builder.context,
        ));

    builder.inputs.extend([
        generate_entity_default_insert_input_object::<E>(builder.context),
        generate_entity_default_update_input_object::<E>(builder.context),
    ]);

    builder.mutations.extend([
        generate_entity_default_create_one_mutation_field::<E, A>(
            builder.context,
            active_model_hooks,
        ),
        generate_entity_default_create_batch_mutation_field::<E, A>(
            builder.context,
            active_model_hooks,
        ),
        generate_entity_default_update_mutation_field::<E, A>(builder.context, active_model_hooks),
        generate_entity_default_delete_mutation_field::<E, A>(builder.context, active_model_hooks),
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
            .map(|rel| RelationBuilder::get_relation(&rel, builder.context))
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
