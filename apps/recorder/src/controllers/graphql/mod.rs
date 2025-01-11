pub mod playground;

use std::collections::HashMap;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::HeaderMap,
    middleware::from_fn_with_state,
    response::Html,
    routing::{get, post},
    Extension,
};
use loco_rs::{app::AppContext, controller::middleware::MiddlewareLayer, prelude::Routes};
use playground::{altair_graphql_playground_asset_middleware, AltairGraphQLPlayground};
use reqwest::header;

use crate::{
    app::AppContextExt,
    auth::{api_auth_middleware, webui_auth_middleware, AuthUserInfo},
};

async fn graphql_playground(header_map: HeaderMap) -> loco_rs::Result<Html<String>> {
    let mut playground_config = AltairGraphQLPlayground::new("/api/graphql");

    if let Some(authorization) = header_map.get(header::AUTHORIZATION) {
        if let Ok(authorization) = authorization.to_str() {
            playground_config.initial_headers = {
                let mut m = HashMap::new();
                m.insert(header::AUTHORIZATION.to_string(), authorization.to_string());
                Some(m)
            }
        }
    }

    let html = Html(playground_config.render("/api/graphql/playground/static/")?);

    Ok(html)
}

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

pub fn routes(state: AppContext) -> Routes {
    Routes::new()
        .prefix("/graphql")
        .add(
            "/playground",
            get(graphql_playground).layer(from_fn_with_state(state.clone(), webui_auth_middleware)),
        )
        .add(
            "/",
            post(graphql_handler).layer(from_fn_with_state(state, api_auth_middleware)),
        )
}

pub fn asset_middlewares() -> Vec<Box<dyn MiddlewareLayer>> {
    vec![Box::new(altair_graphql_playground_asset_middleware(
        "/api/graphql/playground/static/",
    ))]
}
