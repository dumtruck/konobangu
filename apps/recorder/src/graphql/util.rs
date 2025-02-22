use sea_orm::{EntityName, EntityTrait, IdenStatic};
use seaography::BuilderContext;

pub fn get_entity_key<T>(context: &BuilderContext) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    context.entity_object.type_name.as_ref()(<T as EntityName>::table_name(&T::default()))
}

pub fn get_column_key<T>(context: &BuilderContext, column: &T::Column) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_key::<T>(context);
    context.entity_object.column_name.as_ref()(&entity_name, column.as_str())
}

pub fn get_entity_column_key<T>(context: &BuilderContext, column: &T::Column) -> String
where
    T: EntityTrait,
    <T as EntityTrait>::Model: Sync,
{
    let entity_name = get_entity_key::<T>(context);
    let column_name = get_column_key::<T>(context, column);

    format!("{}.{}", &entity_name, &column_name)
}
