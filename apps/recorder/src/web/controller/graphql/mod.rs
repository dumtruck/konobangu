use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Extension, Router, extract::State, middleware::from_fn_with_state, routing::post};

use super::core::Controller;
use crate::{
    app::AppContextTrait,
    auth::{AuthUserInfo, header_www_authenticate_middleware},
    errors::RResult,
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

pub async fn create(ctx: Arc<dyn AppContextTrait>) -> RResult<Controller> {
    let router = Router::<Arc<dyn AppContextTrait>>::new()
        .route("/", post(graphql_handler))
        .layer(from_fn_with_state(ctx, header_www_authenticate_middleware));
    Ok(Controller::from_prefix(CONTROLLER_PREFIX, router))
}
