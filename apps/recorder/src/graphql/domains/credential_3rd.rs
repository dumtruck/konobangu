use std::sync::Arc;

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, InputObject, InputValue, Object, TypeRef,
};
use seaography::{Builder as SeaographyBuilder, BuilderContext};
use serde::{Deserialize, Serialize};
use util_derive::DynamicGraphql;

use crate::{
    app::AppContextTrait,
    auth::AuthUserInfo,
    errors::RecorderError,
    graphql::{
        domains::subscribers::restrict_subscriber_for_entity,
        infra::crypto::{
            register_crypto_column_input_conversion_to_schema_context,
            register_crypto_column_output_conversion_to_schema_context,
        },
    },
    models::credential_3rd,
};

#[derive(DynamicGraphql, Serialize, Deserialize, Clone, Debug)]
struct Credential3rdCheckAvailableInput {
    pub id: i32,
}

impl Credential3rdCheckAvailableInput {
    fn input_type_name() -> &'static str {
        "Credential3rdCheckAvailableInput"
    }

    fn arg_name() -> &'static str {
        "filter"
    }

    fn generate_input_object() -> InputObject {
        InputObject::new(Self::input_type_name())
            .description("The input of the credential3rdCheckAvailable query")
            .field(InputValue::new(
                Credential3rdCheckAvailableInputFieldEnum::Id.as_str(),
                TypeRef::named_nn(TypeRef::INT),
            ))
    }
}

#[derive(DynamicGraphql, Serialize, Deserialize, Clone, Debug)]
pub struct Credential3rdCheckAvailableInfo {
    pub available: bool,
}

impl Credential3rdCheckAvailableInfo {
    fn object_type_name() -> &'static str {
        "Credential3rdCheckAvailableInfo"
    }

    fn generate_output_object() -> Object {
        Object::new(Self::object_type_name())
            .description("The output of the credential3rdCheckAvailable query")
            .field(Field::new(
                Credential3rdCheckAvailableInfoFieldEnum::Available,
                TypeRef::named_nn(TypeRef::BOOLEAN),
                move |ctx| {
                    FieldFuture::new(async move {
                        let subscription_info = ctx.parent_value.try_downcast_ref::<Self>()?;
                        Ok(Some(async_graphql::Value::from(
                            subscription_info.available,
                        )))
                    })
                },
            ))
    }
}

pub fn register_credential3rd_to_schema_context(
    context: &mut BuilderContext,
    ctx: Arc<dyn AppContextTrait>,
) {
    restrict_subscriber_for_entity::<credential_3rd::Entity>(
        context,
        &credential_3rd::Column::SubscriberId,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    register_crypto_column_input_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Password,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Cookies,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx.clone(),
        &credential_3rd::Column::Username,
    );
    register_crypto_column_output_conversion_to_schema_context::<credential_3rd::Entity>(
        context,
        ctx,
        &credential_3rd::Column::Password,
    );
}

pub fn register_credential3rd_to_schema_builder(
    mut builder: SeaographyBuilder,
) -> SeaographyBuilder {
    builder.register_enumeration::<credential_3rd::Credential3rdType>();
    seaography::register_entity!(builder, credential_3rd);

    builder.schema = builder
        .schema
        .register(Credential3rdCheckAvailableInput::generate_input_object());
    builder.schema = builder
        .schema
        .register(Credential3rdCheckAvailableInfo::generate_output_object());

    builder.queries.push(
        Field::new(
            "credential3rdCheckAvailable",
            TypeRef::named_nn(Credential3rdCheckAvailableInfo::object_type_name()),
            move |ctx| {
                FieldFuture::new(async move {
                    let auth_user_info = ctx.data::<AuthUserInfo>()?;
                    let input: Credential3rdCheckAvailableInput = ctx
                        .args
                        .get(Credential3rdCheckAvailableInput::arg_name())
                        .unwrap()
                        .deserialize()?;
                    let app_ctx = ctx.data::<Arc<dyn AppContextTrait>>()?;

                    let credential_model = credential_3rd::Model::find_by_id_and_subscriber_id(
                        app_ctx.as_ref(),
                        input.id,
                        auth_user_info.subscriber_auth.subscriber_id,
                    )
                    .await?
                    .ok_or_else(|| RecorderError::Credential3rdError {
                        message: format!("credential = {} not found", input.id),
                        source: None.into(),
                    })?;

                    let available = credential_model.check_available(app_ctx.as_ref()).await?;
                    Ok(Some(FieldValue::owned_any(
                        Credential3rdCheckAvailableInfo { available },
                    )))
                })
            },
        )
        .argument(InputValue::new(
            Credential3rdCheckAvailableInput::arg_name(),
            TypeRef::named_nn(Credential3rdCheckAvailableInput::input_type_name()),
        )),
    );

    builder
}
