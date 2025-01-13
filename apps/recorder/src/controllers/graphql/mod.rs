use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, middleware::from_fn_with_state, routing::post, Extension};
use loco_rs::{app::AppContext, prelude::Routes};

use crate::{
    app::AppContextExt,
    auth::{api_auth_middleware, AuthUserInfo},
};

async fn graphql_handler(
    State(ctx): State<AppContext>,
    Extension(auth_user_info): Extension<AuthUserInfo>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let graphql_service = ctx.get_graphql_service();

    let mut req = req.into_inner();
    req = req.data(auth_user_info);

    graphql_service.schema.execute(req).await.into()
}

pub fn routes(ctx: Arc<AppContext>) -> Routes {
    Routes::new().prefix("/graphql").add(
        "/",
        post(graphql_handler).layer(from_fn_with_state(ctx, api_auth_middleware)),
    )
}
