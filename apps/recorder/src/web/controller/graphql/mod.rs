use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Extension, Router, extract::State, middleware::from_fn_with_state, routing::post};

use super::core::Controller;
use crate::{
    app::{AppContextTrait, Environment},
    auth::{AuthUserInfo, auth_middleware},
    errors::RecorderResult,
};

pub const CONTROLLER_PREFIX: &str = "/api/graphql";

async fn graphql_handler(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    Extension(auth_user_info): Extension<AuthUserInfo>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_service = ctx.graphql();

    let mut req = req.into_inner();
    req = req.data(auth_user_info);

    graphql_service.schema.execute(req).await.into()
}

// 检查是否是 introspection 查询
fn is_introspection_query(req: &async_graphql::Request) -> bool {
    if let Some(operation) = &req.operation_name
        && operation.starts_with("__")
    {
        return true;
    }

    // 检查查询内容是否包含 introspection 字段
    let query = req.query.as_str();
    query.contains("__schema") || query.contains("__type") || query.contains("__typename")
}

async fn graphql_introspection_handler(
    State(ctx): State<Arc<dyn AppContextTrait>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_service = ctx.graphql();
    let req = req.into_inner();

    if !is_introspection_query(&req) {
        return GraphQLResponse::from(async_graphql::Response::from_errors(vec![
            async_graphql::ServerError::new(
                "Only introspection queries are allowed on this endpoint",
                None,
            ),
        ]));
    }

    graphql_service.schema.execute(req).await.into()
}

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Controller> {
    let mut introspection_handler = post(graphql_introspection_handler);

    if !matches!(ctx.environment(), Environment::Development) {
        introspection_handler =
            introspection_handler.layer(from_fn_with_state(ctx.clone(), auth_middleware));
    }

    let router = Router::<Arc<dyn AppContextTrait>>::new()
        .route(
            "/",
            post(graphql_handler).layer(from_fn_with_state(ctx, auth_middleware)),
        )
        .route("/introspection", introspection_handler);
    Ok(Controller::from_prefix(CONTROLLER_PREFIX, router))
}
